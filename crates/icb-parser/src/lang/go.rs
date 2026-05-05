//! Go language parser using tree-sitter-go.
//!
//! Extracts function declarations, method declarations, call expressions,
//! and type declarations (struct/interface) from Go source files.

use crate::facts::RawNode;
use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::Parser;

use super::common::{child_of_kind, traverse_node};

/// Parse Go source code and return the extracted facts.
pub fn parse_go(source: &str) -> Result<Vec<RawNode>, IcbError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::language())
        .map_err(|e| IcbError::Parse(format!("cannot set tree-sitter-go language: {e}")))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| IcbError::Parse("tree-sitter parse returned None for Go source".into()))?;

    let mut facts = Vec::new();

    let classifier =
        |node: &tree_sitter::Node, source: &str| -> Option<(NodeKind, Option<String>, bool)> {
            match node.kind() {
                "function_declaration" | "method_declaration" => {
                    let name = child_of_kind(*node, "identifier")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    Some((NodeKind::Function, name, true))
                }
                "type_declaration" => {
                    if let Some(type_spec) = child_of_kind(*node, "type_spec") {
                        if child_of_kind(type_spec, "struct_type").is_some()
                            || child_of_kind(type_spec, "interface_type").is_some()
                        {
                            let name = child_of_kind(type_spec, "identifier")
                                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                                .map(|s| s.to_string());
                            return Some((NodeKind::Class, name, true));
                        }
                    }
                    None
                }
                "call_expression" => {
                    let name_node = child_of_kind(*node, "identifier")
                        .or_else(|| child_of_kind(*node, "selector_expression"));
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
        Language::Go,
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
        let code = "package main\nfunc foo() {}\n";
        let facts = parse_go(code).unwrap();
        let funcs: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_method() {
        let code = "package main\ntype S struct{}\nfunc (s S) bar() {}\n";
        let facts = parse_go(code).unwrap();
        let methods: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert!(methods.iter().any(|m| m.name.as_deref() == Some("bar")));
    }

    #[test]
    fn test_call_expression() {
        let code = "package main\nfunc baz() { foo() }\n";
        let facts = parse_go(code).unwrap();
        let calls: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::CallSite)
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_struct_type() {
        let code = "package main\ntype MyStruct struct {}\n";
        let facts = parse_go(code).unwrap();
        let classes: Vec<_> = facts.iter().filter(|n| n.kind == NodeKind::Class).collect();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name.as_deref(), Some("MyStruct"));
    }
}
