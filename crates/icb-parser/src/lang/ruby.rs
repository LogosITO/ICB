//! Ruby language parser using tree-sitter-ruby.
//!
//! Extracts method definitions, singleton methods, class/module definitions,
//! call expressions, lambdas, and blocks as anonymous functions.

use crate::facts::RawNode;
use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::Parser;

use super::common::{child_of_kind, traverse_node};

/// Parse Ruby source code and return the extracted facts.
pub fn parse_ruby(source: &str) -> Result<Vec<RawNode>, IcbError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_ruby::language())
        .map_err(|e| IcbError::Parse(format!("cannot set tree-sitter-ruby language: {e}")))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| IcbError::Parse("tree-sitter parse returned None for Ruby source".into()))?;

    let mut facts = Vec::new();

    let classifier = |node: &tree_sitter::Node,
                      source: &str|
     -> Option<(NodeKind, Option<String>, bool)> {
        match node.kind() {
            "method" | "singleton_method" => {
                let name = child_of_kind(*node, "identifier")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());
                Some((NodeKind::Function, name, true))
            }
            "class" | "module" => {
                let name = child_of_kind(*node, "constant")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());
                Some((NodeKind::Class, name, true))
            }
            "call" => {
                let name_node = child_of_kind(*node, "identifier")
                    .or_else(|| child_of_kind(*node, "constant"))
                    .or_else(|| child_of_kind(*node, "method_identifier"));
                let name = name_node
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .map(|s| s.to_string());
                Some((NodeKind::CallSite, name, false))
            }
            "lambda" => Some((NodeKind::Function, Some("lambda".into()), true)),
            "do_block" | "brace_block" => Some((NodeKind::Function, Some("block".into()), true)),
            _ => None,
        }
    };

    traverse_node(
        tree.root_node(),
        source,
        &mut facts,
        None,
        Language::Ruby,
        &classifier,
    );
    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use icb_common::NodeKind;

    #[test]
    fn test_simple_method() {
        let code = "def foo; end";
        let facts = parse_ruby(code).unwrap();
        let funcs: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_class() {
        let code = "class MyClass; end";
        let facts = parse_ruby(code).unwrap();
        let classes: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Class && n.name.as_deref() == Some("MyClass"))
            .collect();
        assert!(!classes.is_empty(), "expected class MyClass");
    }

    #[test]
    fn test_call() {
        let code = "puts 'hello'";
        let facts = parse_ruby(code).unwrap();
        let calls: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::CallSite)
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name.as_deref(), Some("puts"));
    }

    #[test]
    fn test_lambda() {
        let code = "-> { }";
        let facts = parse_ruby(code).unwrap();
        let lambdas: Vec<_> = facts
            .iter()
            .filter(|n| n.name.as_deref() == Some("lambda"))
            .collect();
        assert!(!lambdas.is_empty(), "expected at least one lambda");
    }
}
