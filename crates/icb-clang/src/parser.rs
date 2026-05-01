//! High‑performance Clang AST visitor that extracts ICB facts for the
//! Code Property Graph.
//!
//! # Overview
//!
//! The parser translates a single C/C++ translation unit (TU) into a flat
//! vector of [`RawNode`] values.  It traverses the Clang AST recursively,
//! but **only materialises nodes that are relevant for call‑graph
//! construction**:
//!
//! * Functions and methods (`Function`),
//! * Classes, structs (`Class`),
//! * Call expressions (`CallSite`),
//! * Variable declarations (`Variable`),
//! * Parameter declarations (`Parameter`).
//!
//! All other AST constructs – compound statements, binary operators,
//! implicit casts – are transparently skipped.  This keeps memory usage
//! linear in the number of interesting declarations instead of quadratic in
//! the AST depth.
//!
//! # System header isolation
//!
//! When `allow_system` is `false`, any cursor whose location Clang considers
//! a system header is dropped together with its entire subtree.  This
//! removes thousands of standard‑library nodes at almost zero cost and is
//! enabled by default in the CLI and server.
//!
//! # Memory safety
//!
//! Every `CXString` obtained from the Clang API is disposed via
//! `clang_disposeString` immediately after its contents have been copied to
//! a Rust `String`.  Temporary C strings created for command‑line arguments
//! are converted back to owned `CString` and dropped when no longer needed.
//!
//! # Concurrency
//!
//! A single `parse_cpp_file` call processes one TU on the calling thread.
//! Parallelism is achieved by processing multiple TUs concurrently at the
//! project level (see [`super::project`]).
//!
//! # Limitations
//!
//! * The parser does **not** resolve overloaded functions – it stores the
//!   spelling of the call expression or its referenced entity as‑is.
//! * Template instantiations are visited as part of the parent TU, which may
//!   cause the same function body to be processed multiple times.
//! * Precompiled headers are not supported.

#![allow(non_upper_case_globals)]

use clang_sys::*;
use icb_common::{IcbError, Language, NodeKind};
use icb_parser::facts::RawNode;
use std::ffi::{c_uint, CString};
use std::os::raw::c_void;
use std::ptr;
use tempfile::Builder;

/// Returns `true` when the given cursor resides in a file that Clang
/// treats as a system header.
///
/// System headers typically include standard library files (e.g.
/// `/usr/include`, MSVC SDK paths).  This function is used to decide
/// whether a subtree should be excluded from the fact list.
fn is_in_system_header(cursor: CXCursor) -> bool {
    unsafe { clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)) != 0 }
}

/// Returns the absolute path of the source file that contains `cursor`,
/// or `None` if the information is unavailable.
///
/// The Clang API returns a `CXString` which is copied into an owned `String`
/// and then immediately disposed of via [`clang_disposeString`].
fn cursor_file(cursor: CXCursor) -> Option<String> {
    unsafe {
        let loc = clang_getCursorLocation(cursor);
        let mut file: CXFile = ptr::null_mut();
        clang_getFileLocation(
            loc,
            &mut file,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        if file.is_null() {
            return None;
        }
        let name_cx = clang_getFileName(file);
        let cstr = clang_getCString(name_cx);
        let result = if cstr.is_null() {
            None
        } else {
            Some(
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned(),
            )
        };
        clang_disposeString(name_cx);
        result
    }
}

/// Parse a single C/C++ source file and return the extracted facts.
///
/// The source is written to a temporary `.cpp` file and Clang is invoked
/// with the supplied `args` (e.g. `["-std=c++17", "-Iinclude"]`).
///
/// # Arguments
///
/// * `source` – The raw source code.
/// * `args` – Command‑line arguments passed to the Clang frontend.
/// * `_file_name` – Ignored in this implementation; preserved for API
///   compatibility.
/// * `allow_system` – If `false`, nodes from system headers are excluded.
///
/// # Errors
///
/// Returns [`IcbError::Parse`] when the Clang index cannot be created or the
/// translation unit fails to parse.
pub fn parse_cpp_file(
    source: &str,
    args: &[String],
    _file_name: Option<&str>,
    allow_system: bool,
) -> Result<Vec<RawNode>, IcbError> {
    let index = unsafe { clang_createIndex(0, 0) };
    if index.is_null() {
        return Err(IcbError::Parse("failed to create Clang index".into()));
    }

    let temp_file = Builder::new()
        .suffix(".cpp")
        .tempfile()
        .map_err(|e| IcbError::Parse(format!("tempfile error: {}", e)))?;
    std::fs::write(temp_file.path(), source).map_err(IcbError::Io)?;
    let filename = temp_file
        .path()
        .to_str()
        .ok_or_else(|| IcbError::Parse("non-UTF8 temp path".into()))?;
    let filename_c = CString::new(filename).unwrap();

    let arg_ptrs: Vec<*const i8> = args
        .iter()
        .map(|a| CString::new(a.as_str()).unwrap().into_raw() as *const i8)
        .collect();
    let mut tu: CXTranslationUnit = ptr::null_mut();

    let error = unsafe {
        clang_parseTranslationUnit2(
            index,
            filename_c.as_ptr(),
            arg_ptrs.as_ptr(),
            args.len() as i32,
            ptr::null_mut(),
            0,
            CXTranslationUnit_None,
            &mut tu,
        )
    };

    // Reclaim the CString allocations.
    for &cstr_ptr in &arg_ptrs {
        unsafe {
            let _ = CString::from_raw(cstr_ptr as *mut i8);
        }
    }

    if error != CXError_Success {
        unsafe { clang_disposeIndex(index) };
        return Err(IcbError::Parse(format!(
            "failed to parse translation unit, error code {:?}",
            error
        )));
    }

    let cursor = unsafe { clang_getTranslationUnitCursor(tu) };
    let mut nodes = Vec::new();
    visit_children(cursor, &mut nodes, None, false, allow_system);

    unsafe {
        clang_disposeTranslationUnit(tu);
        clang_disposeIndex(index);
    }

    Ok(nodes)
}

/// Mutable state passed through the recursive AST visitor.
///
/// This struct is allocated on the stack and a raw pointer is handed to the
/// C callback.  It is safe because the callback is guaranteed to run on the
/// same thread and does not outlive the struct.
struct VisitorContext<'a> {
    /// The accumulated list of facts.
    nodes: &'a mut Vec<RawNode>,
    /// The index of the most recently added node that serves as a parent for
    /// siblings, or `None` at the top level.
    latest_parent: Option<usize>,
    /// Whether an ancestor is located in a system header.
    in_system: bool,
    /// Whether system‑header nodes are allowed at all.
    allow_system: bool,
}

/// Recursively walk the AST rooted at `cursor`.
///
/// Returns the index of the node that should be used as the parent for the
/// next sibling.  If the visited node is **not** a container (variable,
/// parameter, call expression), the original `parent_idx` is returned so
/// that the flat list remains shallow.
fn visit_children(
    cursor: CXCursor,
    nodes: &mut Vec<RawNode>,
    parent_idx: Option<usize>,
    in_system: bool,
    allow_system: bool,
) -> Option<usize> {
    let is_sys = is_in_system_header(cursor);

    if !allow_system && is_sys {
        return parent_idx;
    }
    if in_system && is_sys {
        return parent_idx;
    }

    let kind = unsafe { clang_getCursorKind(cursor) };
    let (node_kind, name, usr, is_container) = match kind {
        CXCursor_FunctionDecl | CXCursor_CXXMethod => (
            NodeKind::Function,
            Some(cursor_spelling(cursor)),
            Some(cursor_usr(cursor)),
            true,
        ),
        CXCursor_ClassDecl | CXCursor_StructDecl => (
            NodeKind::Class,
            Some(cursor_spelling(cursor)),
            Some(cursor_usr(cursor)),
            true,
        ),
        CXCursor_CallExpr => {
            let referenced = unsafe { clang_getCursorReferenced(cursor) };
            let spelling = if referenced.kind == CXCursor_InvalidFile {
                cursor_spelling(cursor)
            } else {
                cursor_spelling(referenced)
            };
            (NodeKind::CallSite, Some(spelling), None, false)
        }
        CXCursor_VarDecl => (
            NodeKind::Variable,
            Some(cursor_spelling(cursor)),
            None,
            false,
        ),
        CXCursor_ParmDecl => (
            NodeKind::Parameter,
            Some(cursor_spelling(cursor)),
            None,
            false,
        ),
        _ => {
            // Transparent node – descend into children without creating a fact.
            let mut ctx = VisitorContext {
                nodes,
                latest_parent: parent_idx,
                in_system,
                allow_system,
            };
            let ctx_ptr = &mut ctx as *mut VisitorContext as *mut c_void;
            unsafe {
                clang_visitChildren(cursor, visitor_callback, ctx_ptr);
            }
            return ctx.latest_parent;
        }
    };

    let (start_line, start_col, end_line, end_col) = cursor_location(cursor);

    let idx = nodes.len();
    nodes.push(RawNode {
        language: Language::Cpp,
        kind: node_kind,
        name,
        usr,
        start_line,
        start_col,
        end_line,
        end_col,
        children: Vec::new(),
        source_file: cursor_file(cursor),
    });

    if let Some(pidx) = parent_idx {
        nodes[pidx].children.push(idx);
    }

    if !is_container {
        return parent_idx;
    }

    let next_in_system = in_system || is_sys;
    let mut ctx = VisitorContext {
        nodes,
        latest_parent: Some(idx),
        in_system: next_in_system,
        allow_system,
    };
    let ctx_ptr = &mut ctx as *mut VisitorContext as *mut c_void;
    unsafe {
        clang_visitChildren(cursor, visitor_callback, ctx_ptr);
    }

    ctx.latest_parent
}

/// C callback for [`clang_visitChildren`].
///
/// Converts the opaque client data pointer back to a [`VisitorContext`] and
/// delegates to [`visit_children`].
extern "C" fn visitor_callback(
    cursor: CXCursor,
    _parent: CXCursor,
    client_data: CXClientData,
) -> CXChildVisitResult {
    let ctx: &mut VisitorContext = unsafe { &mut *(client_data as *mut VisitorContext) };
    ctx.latest_parent = visit_children(
        cursor,
        ctx.nodes,
        ctx.latest_parent,
        ctx.in_system,
        ctx.allow_system,
    );
    CXChildVisit_Continue
}

/// Returns the 1‑based line and column coordinates of `cursor`.
///
/// The tuple is `(start_line, start_column, end_line, end_column)`.
fn cursor_location(cursor: CXCursor) -> (usize, usize, usize, usize) {
    let range = unsafe { clang_getCursorExtent(cursor) };
    let start = unsafe { clang_getRangeStart(range) };
    let end = unsafe { clang_getRangeEnd(range) };

    let mut line: c_uint = 0;
    let mut column: c_uint = 0;

    unsafe {
        clang_getPresumedLocation(start, ptr::null_mut(), &mut line, &mut column);
    }
    let s_line = line as usize;
    let s_col = column as usize;

    unsafe {
        clang_getPresumedLocation(end, ptr::null_mut(), &mut line, &mut column);
    }
    let e_line = line as usize;
    let e_col = column as usize;

    (s_line, s_col, e_line, e_col)
}

/// Returns the spelling of the given cursor as a Rust [`String`].
///
/// The underlying `CXString` is properly disposed.
fn cursor_spelling(cursor: CXCursor) -> String {
    unsafe {
        let cxstring = clang_getCursorSpelling(cursor);
        let s = clang_getCString(cxstring);
        let result = if s.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned()
        };
        clang_disposeString(cxstring);
        result
    }
}

/// Returns the Unified Symbol Resolution (USR) string for the given cursor.
///
/// The USR is a persistent identifier that uniquely identifies the entity.
/// The underlying `CXString` is properly disposed.
fn cursor_usr(cursor: CXCursor) -> String {
    unsafe {
        let cxstring = clang_getCursorUSR(cursor);
        let s = clang_getCString(cxstring);
        let result = if s.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned()
        };
        clang_disposeString(cxstring);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_function() {
        let code = "void foo() {}";
        let facts = parse_cpp_file(code, &[], None, true).unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].kind, NodeKind::Function);
        assert_eq!(facts[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn parse_function_with_call() {
        let code = "void bar() {} void baz() { bar(); }";
        let facts = parse_cpp_file(code, &[], None, true).unwrap();
        let calls: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::CallSite)
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name.as_deref(), Some("bar"));
    }

    #[test]
    fn parse_class_with_method() {
        let code = "class A { void f() {} };";
        let facts = parse_cpp_file(code, &[], None, true).unwrap();
        let classes: Vec<_> = facts.iter().filter(|n| n.kind == NodeKind::Class).collect();
        assert_eq!(classes.len(), 1);
        let methods: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(methods.len(), 1);
    }

    #[test]
    fn exclude_system_headers() {
        let code = "#include <vector>\nvoid func() {}";
        let facts = parse_cpp_file(code, &[], None, false).unwrap();
        assert!(facts.iter().all(|n| n.kind == NodeKind::Function));
    }
}
