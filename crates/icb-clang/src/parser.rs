#![allow(non_upper_case_globals)]

use clang_sys::*;
use icb_common::{IcbError, Language, NodeKind};
use icb_parser::facts::RawNode;
use std::ffi::{c_uint, CString};
use std::os::raw::c_void;
use std::ptr;
use tempfile::Builder;

/// Returns `true` if the cursor is located in a system header.
fn is_in_system_header(cursor: CXCursor) -> bool {
    unsafe { clang_Location_isInSystemHeader(clang_getCursorLocation(cursor)) != 0 }
}

/// Returns the file name that contains the given cursor.
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
        let name = clang_getFileName(file);
        let cstr = clang_getCString(name);
        if cstr.is_null() {
            None
        } else {
            Some(
                std::ffi::CStr::from_ptr(cstr)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

/// Parse C/C++ source code using Clang and return a flat list of facts.
///
/// The source is written to a temporary file and Clang is invoked with `args`.
/// If `allow_system` is `false`, nodes originating from system headers are
/// excluded from the result.
///
/// # Errors
///
/// Returns [`IcbError::Parse`] if the Clang API cannot be initialised or
/// the source cannot be parsed.
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

struct VisitorContext<'a> {
    nodes: &'a mut Vec<RawNode>,
    latest_parent: Option<usize>,
    in_system: bool,
    allow_system: bool,
}

fn visit_children(
    cursor: CXCursor,
    nodes: &mut Vec<RawNode>,
    parent_idx: Option<usize>,
    in_system: bool,
    allow_system: bool,
) -> Option<usize> {
    let is_sys = is_in_system_header(cursor);

    // If system headers are disabled, skip the whole sub‑tree.
    if !allow_system && is_sys {
        return parent_idx;
    }

    // If we are already inside a system header, do not recurse further.
    if in_system && is_sys {
        return parent_idx;
    }

    let kind = unsafe { clang_getCursorKind(cursor) };
    let (node_kind, name, usr) = match kind {
        CXCursor_FunctionDecl => (
            NodeKind::Function,
            Some(cursor_spelling(cursor)),
            Some(cursor_usr(cursor)),
        ),
        CXCursor_CXXMethod => (
            NodeKind::Function,
            Some(cursor_spelling(cursor)),
            Some(cursor_usr(cursor)),
        ),
        CXCursor_ClassDecl | CXCursor_StructDecl => (
            NodeKind::Class,
            Some(cursor_spelling(cursor)),
            Some(cursor_usr(cursor)),
        ),
        CXCursor_CallExpr => {
            let referenced = unsafe { clang_getCursorReferenced(cursor) };
            let name = if referenced.kind == CXCursor_InvalidFile {
                cursor_spelling(cursor)
            } else {
                cursor_spelling(referenced)
            };
            (NodeKind::CallSite, Some(name), None)
        }
        CXCursor_VarDecl => (NodeKind::Variable, Some(cursor_spelling(cursor)), None),
        CXCursor_ParmDecl => (NodeKind::Parameter, Some(cursor_spelling(cursor)), None),
        _ => {
            let mut ctx = VisitorContext {
                nodes,
                latest_parent: parent_idx,
                in_system,
                allow_system,
            };
            let ctx_ptr: *mut c_void = &mut ctx as *mut _ as *mut c_void;
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

    let next_in_system = in_system || is_sys;

    let mut ctx = VisitorContext {
        nodes,
        latest_parent: Some(idx),
        in_system: next_in_system,
        allow_system,
    };
    let ctx_ptr: *mut c_void = &mut ctx as *mut _ as *mut c_void;
    unsafe {
        clang_visitChildren(cursor, visitor_callback, ctx_ptr);
    }

    ctx.latest_parent
}

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
