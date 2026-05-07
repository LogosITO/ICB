//! Rust language parser using tree-sitter-rust.
//!
//! Extracts function declarations, method declarations, call expressions,
//! and trait/struct/enum definitions from Rust source files.

use crate::facts::RawNode;
use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::Parser;

use super::common::{child_of_kind, traverse_node};

/// Parse Rust source code and return the extracted facts.
pub fn parse_rust(source: &str) -> Result<Vec<RawNode>, IcbError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .map_err(|e| IcbError::Parse(format!("cannot set tree-sitter-rust language: {e}")))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| IcbError::Parse("tree-sitter parse returned None for Rust source".into()))?;

    let mut facts = Vec::new();

    let classifier =
        |node: &tree_sitter::Node, source: &str| -> Option<(NodeKind, Option<String>, bool)> {
            match node.kind() {
                "function_item" | "function_signature_item" => {
                    let name = child_of_kind(*node, "identifier")
                        .or_else(|| child_of_kind(*node, "name"))
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    Some((NodeKind::Function, name, true))
                }
                "impl_item" | "trait_item" | "struct_item" | "enum_item" | "union_item" => {
                    let name = child_of_kind(*node, "type_identifier")
                        .or_else(|| child_of_kind(*node, "identifier"))
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    Some((NodeKind::Class, name, true))
                }
                "call_expression" | "macro_invocation" => {
                    let name_node = child_of_kind(*node, "identifier")
                        .or_else(|| child_of_kind(*node, "field_expression"))
                        .or_else(|| child_of_kind(*node, "scoped_identifier"));
                    let name = name_node
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    Some((NodeKind::CallSite, name, false))
                }
                _ => None,
            }
        };

    traverse_node(
        tree.root_node(),
        source,
        &mut facts,
        None,
        Language::Rust,
        &classifier,
    );
    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use icb_common::NodeKind;

    #[test]
    fn test_simple_function() {
        let code = "fn foo() {}";
        let facts = parse_rust(code).unwrap();
        let funcs: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_method_in_impl() {
        let code = "struct S; impl S { fn bar(&self) {} }";
        let facts = parse_rust(code).unwrap();
        let methods: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function && n.name.as_deref() == Some("bar"))
            .collect();
        assert!(!methods.is_empty());
    }

    #[test]
    fn test_call_expression() {
        let code = "fn baz() { foo(); }";
        let facts = parse_rust(code).unwrap();
        let calls: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::CallSite)
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_struct_type() {
        let code = "struct MyStruct {}";
        let facts = parse_rust(code).unwrap();
        let classes: Vec<_> = facts.iter().filter(|n| n.kind == NodeKind::Class).collect();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name.as_deref(), Some("MyStruct"));
    }
}
