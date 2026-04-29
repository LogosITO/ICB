#![allow(non_upper_case_globals)]

use clang_sys::*;
use icb_common::{IcbError, Language, NodeKind};
use icb_parser::facts::RawNode;
use std::ffi::{c_uint, CString};
use std::os::raw::c_void;
use std::ptr;
use tempfile::Builder;

pub fn parse_cpp_file(source: &str, args: &[String]) -> Result<Vec<RawNode>, IcbError> {
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

    let mut arg_ptrs: Vec<*const i8> = args
        .iter()
        .map(|a| CString::new(a.as_str()).unwrap().into_raw() as *const i8)
        .collect();
    arg_ptrs.push(ptr::null());

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

    for &cstr_ptr in &arg_ptrs[..args.len()] {
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
    visit_children(cursor, &mut nodes, None);

    unsafe {
        clang_disposeTranslationUnit(tu);
        clang_disposeIndex(index);
    }

    Ok(nodes)
}

struct VisitorContext<'a> {
    nodes: &'a mut Vec<RawNode>,
    latest_parent: Option<usize>,
}

fn visit_children(
    cursor: CXCursor,
    nodes: &mut Vec<RawNode>,
    parent_idx: Option<usize>,
) -> Option<usize> {
    let kind = unsafe { clang_getCursorKind(cursor) };
    let (node_kind, name, usr) = match kind {
        CXCursor_FunctionDecl => {
            let name = cursor_spelling(cursor);
            let usr = cursor_usr(cursor);
            (NodeKind::Function, Some(name), Some(usr))
        }
        CXCursor_CXXMethod => {
            let name = cursor_spelling(cursor);
            let usr = cursor_usr(cursor);
            (NodeKind::Function, Some(name), Some(usr))
        }
        CXCursor_ClassDecl | CXCursor_StructDecl => {
            let name = cursor_spelling(cursor);
            let usr = cursor_usr(cursor);
            (NodeKind::Class, Some(name), Some(usr))
        }
        CXCursor_CallExpr => {
            let referenced = unsafe { clang_getCursorReferenced(cursor) };
            let name = if referenced.kind == CXCursor_InvalidFile {
                cursor_spelling(cursor)
            } else {
                cursor_spelling(referenced)
            };
            (NodeKind::CallSite, Some(name), None)
        }
        CXCursor_VarDecl => {
            let name = cursor_spelling(cursor);
            (NodeKind::Variable, Some(name), None)
        }
        CXCursor_ParmDecl => {
            let name = cursor_spelling(cursor);
            (NodeKind::Parameter, Some(name), None)
        }
        _ => {
            let mut ctx = VisitorContext {
                nodes,
                latest_parent: parent_idx,
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
    });

    if let Some(pidx) = parent_idx {
        nodes[pidx].children.push(idx);
    }

    let new_parent = Some(idx);

    let mut ctx = VisitorContext {
        nodes,
        latest_parent: new_parent,
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
    ctx.latest_parent = visit_children(cursor, ctx.nodes, ctx.latest_parent);
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
    let start_line = line as usize;
    let start_col = column as usize;

    unsafe {
        clang_getPresumedLocation(end, ptr::null_mut(), &mut line, &mut column);
    }
    let end_line = line as usize;
    let end_col = column as usize;

    (start_line, start_col, end_line, end_col)
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

#[cfg(test)]
mod tests {
    use super::*;
    use icb_common::NodeKind;

    #[test]
    fn test_parse_cpp_with_flags() {
        let source = "#ifndef FOO\n#define FOO 1\n#endif\nint val = FOO;";
        let args = vec!["-std=c++17".to_string()];
        let facts = parse_cpp_file(source, &args).expect("parsing should succeed");
        let vars: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Variable)
            .collect();
        assert!(!vars.is_empty());
        assert!(vars.iter().any(|v| v.name.as_deref() == Some("val")));
    }
}
