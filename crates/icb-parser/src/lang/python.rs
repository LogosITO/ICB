//! Python language parser using tree-sitter-python.
//!
//! Extracts function definitions (including `async def`), class definitions,
//! call expressions, lambdas as anonymous functions, and optionally
//! identifiers as variables.

use crate::facts::RawNode;
use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::Parser;

use super::common::traverse_node;

/// Parse Python source code and return the extracted facts.
///
/// By default variables (identifiers) are **not** included to keep the
/// graph small.  Use [`parse_python_detailed`] if you need them.
pub fn parse_python(source: &str) -> Result<Vec<RawNode>, IcbError> {
    parse_python_impl(source, false)
}

/// Parse Python source code and return facts **including** variable nodes.
pub fn parse_python_detailed(source: &str) -> Result<Vec<RawNode>, IcbError> {
    parse_python_impl(source, true)
}

fn parse_python_impl(source: &str, include_variables: bool) -> Result<Vec<RawNode>, IcbError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::language())
        .map_err(|e| IcbError::Parse(format!("cannot set tree-sitter-python language: {e}")))?;

    let tree = parser.parse(source, None).ok_or_else(|| {
        IcbError::Parse("tree-sitter parse returned None for Python source".into())
    })?;

    let mut facts = Vec::new();

    let classifier =
        move |node: &tree_sitter::Node, source: &str| -> Option<(NodeKind, Option<String>, bool)> {
            match node.kind() {
                "function_definition" | "async_function_definition" => {
                    let name = node
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    Some((NodeKind::Function, name, true))
                }
                "class_definition" => {
                    let name = node
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    Some((NodeKind::Class, name, true))
                }
                "call" => {
                    let name = node
                        .child_by_field_name("function")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    Some((NodeKind::CallSite, name, false))
                }
                "lambda" => Some((NodeKind::Function, Some("lambda".into()), true)),
                "identifier" if include_variables => {
                    let name = node
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(|s| s.to_string());
                    Some((NodeKind::Variable, name, false))
                }
                _ => None,
            }
        };

    traverse_node(
        tree.root_node(),
        source,
        &mut facts,
        None,
        Language::Python,
        &classifier,
    );
    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use icb_common::NodeKind;

    #[test]
    fn test_parse_simple_function() {
        let source = "def hello(): pass";
        let facts = parse_python(source).expect("parsing should succeed");
        let functions: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name.as_deref(), Some("hello"));
    }

    #[test]
    fn test_parse_nested_function() {
        let source = "def outer():\n    def inner(): pass";
        let facts = parse_python(source).expect("parsing should succeed");
        let outer = facts.iter().find(|n| n.name.as_deref() == Some("outer"));
        let inner = facts.iter().find(|n| n.name.as_deref() == Some("inner"));
        assert!(outer.is_some(), "outer function not found");
        assert!(inner.is_some(), "inner function not found");
    }

    #[test]
    fn test_call_site_has_name() {
        let source = "foo()";
        let facts = parse_python(source).expect("parsing should succeed");
        let calls: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::CallSite)
            .collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_async_function() {
        let source = "async def bar(): pass";
        let facts = parse_python(source).unwrap();
        let funcs: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name.as_deref(), Some("bar"));
    }

    #[test]
    fn test_lambda() {
        let source = "lambda x: x";
        let facts = parse_python(source).unwrap();
        // Лямбда создаёт несколько фактов (сама лямбда + её тело)
        let lambdas: Vec<_> = facts
            .iter()
            .filter(|n| n.name.as_deref() == Some("lambda"))
            .collect();
        assert!(!lambdas.is_empty(), "expected at least one lambda");
    }

    #[test]
    fn test_include_variables() {
        let source = "x = 1";
        let facts = parse_python_detailed(source).unwrap();
        let vars: Vec<_> = facts
            .iter()
            .filter(|n| n.kind == NodeKind::Variable)
            .collect();
        assert!(!vars.is_empty());
        let facts_no_vars = parse_python(source).unwrap();
        assert!(!facts_no_vars.iter().any(|n| n.kind == NodeKind::Variable));
    }
}
