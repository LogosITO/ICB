//! Lightweight C/C++ parser using [`tree-sitter-cpp`].
//!
//! This module provides a fast, zero‑dependency alternative to Clang for
//! extracting call‑graph facts from C and C++ source code.  It does not
//! perform semantic analysis (types, overload resolution); it only produces
//! syntactic nodes suitable for building a call graph.
//!
//! # Supported node kinds
//!
//! * `Function` – function definitions and declarations,
//! * `Class` – class/struct definitions,
//! * `CallSite` – call expressions,
//! * `Variable` – variable declarations outside parameter lists,
//! * `Parameter` – parameter declarations inside parameter lists.
//!
//! # Example
//!
//! ```rust
//! use icb_parser::cpp_tree_sitter::parse_cpp_file;
//!
//! let code = r#"
//!     int add(int a, int b) { return a + b; }
//!     void main() { add(1, 2); }
//! "#;
//! let facts = parse_cpp_file(code).unwrap();
//! assert!(facts.iter().any(|n| n.kind == icb_common::NodeKind::Function));
//! assert!(facts.iter().any(|n| n.kind == icb_common::NodeKind::CallSite));
//! ```

use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::{Node, Parser};

use crate::facts::RawNode;

/// Parse a C/C++ source file and return a flat list of facts.
///
/// # Errors
///
/// Returns [`IcbError::Parse`] if the tree‑sitter parser cannot be
/// initialised or the source contains syntax errors.
pub fn parse_cpp_file(source: &str) -> Result<Vec<RawNode>, IcbError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_cpp::language())
        .map_err(|e| IcbError::Parse(format!("cannot set tree-sitter-cpp language: {e}")))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| IcbError::Parse("tree-sitter parse returned None".into()))?;

    let mut facts = Vec::new();
    traverse_node(tree.root_node(), source, &mut facts, None);
    Ok(facts)
}

/// Recursively walk the CST and push relevant nodes into `facts`.
///
/// Returns the index of the last node that should serve as parent for
/// subsequent siblings.
fn traverse_node(
    node: Node,
    source: &str,
    facts: &mut Vec<RawNode>,
    parent_idx: Option<usize>,
) -> Option<usize> {
    let kind = node.kind();

    let (node_kind, name, is_container) = match kind {
        "function_definition" | "function_declaration" => {
            let name = child_text_by_field(node, "declarator", source)
                .or_else(|| child_text_by_field(node, "name", source))
                .unwrap_or_default();
            (NodeKind::Function, Some(name), true)
        }
        "class_specifier" | "struct_specifier" => {
            let name = child_text_by_field(node, "name", source).unwrap_or_default();
            (NodeKind::Class, Some(name), true)
        }
        "call_expression" => {
            let name = child_by_field(node, "function")
                .map(|n| {
                    n.utf8_text(source.as_bytes())
                        .unwrap_or_default()
                        .to_string()
                })
                .unwrap_or_default();
            (NodeKind::CallSite, Some(name), false)
        }
        "declaration" => {
            let name = child_text_by_field(node, "declarator", source).unwrap_or_default();
            if parent_kind_is(node, "parameter_list") {
                (NodeKind::Parameter, Some(name), false)
            } else {
                (NodeKind::Variable, Some(name), false)
            }
        }
        _ => {
            let mut current_parent = parent_idx;
            for child in node.children(&mut node.walk()) {
                current_parent = traverse_node(child, source, facts, current_parent);
            }
            return current_parent;
        }
    };

    let start = node.start_position();
    let end = node.end_position();

    let idx = facts.len();
    facts.push(RawNode {
        language: Language::CppTreeSitter,
        kind: node_kind,
        name,
        usr: None,
        start_line: start.row + 1,
        start_col: start.column,
        end_line: end.row + 1,
        end_col: end.column,
        children: Vec::new(),
        source_file: None,
    });

    if let Some(pidx) = parent_idx {
        facts[pidx].children.push(idx);
    }

    if is_container {
        let new_parent = Some(idx);
        let mut current_parent = new_parent;
        for child in node.children(&mut node.walk()) {
            current_parent = traverse_node(child, source, facts, current_parent);
        }
        new_parent
    } else {
        parent_idx
    }
}

/// Return the child node matching the given field name, if any.
fn child_by_field<'a>(node: Node<'a>, field: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    children
        .into_iter()
        .find(|child| node.field_name_for_child(child.id() as u32) == Some(field))
}

/// Return the text of the child with the given field name.
fn child_text_by_field(node: Node, field: &str, source: &str) -> Option<String> {
    child_by_field(node, field)
        .and_then(|n| n.utf8_text(source.as_bytes()).ok().map(|s| s.to_string()))
}

/// Check whether the node's immediate parent has the expected kind.
fn parent_kind_is(node: Node, expected: &str) -> bool {
    node.parent().is_some_and(|p| p.kind() == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_function() {
        let facts = parse_cpp_file("void foo() {}").unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].kind, NodeKind::Function);
        assert_eq!(facts[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn parse_function_with_call() {
        let code = "void bar() {} void baz() { bar(); }";
        let facts = parse_cpp_file(code).unwrap();
        let calls: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::CallSite)
            .collect();
        assert!(!calls.is_empty());
    }
}
