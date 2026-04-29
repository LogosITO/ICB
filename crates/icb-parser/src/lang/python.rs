use crate::facts::RawNode;
use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::Parser;

/// Parse Python source code and return a list of `RawNode` facts.
///
/// This function uses a tree-sitter Python parser to walk the CST and
/// extract function/class definitions, calls, and identifiers. Nodes that
/// are not relevant for the graph are skipped.
///
/// # Errors
///
/// Returns [`IcbError::Parse`] if the tree-sitter parser cannot be
/// initialised or if parsing fails.
///
/// # Examples
///
/// ```rust
/// use icb_parser::lang::python::parse_python;
///
/// let source = "def answer(): return 42";
/// let facts = parse_python(source).expect("valid Python");
/// // There should be at least a function node and a return statement.
/// assert!(facts.iter().any(|n| n.name.as_deref() == Some("answer")));
/// ```
pub fn parse_python(source: &str) -> Result<Vec<RawNode>, IcbError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::language())
        .map_err(|e| IcbError::Parse(e.to_string()))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| IcbError::Parse("parse failed".into()))?;

    let mut nodes = Vec::new();
    collect_nodes(&tree.root_node(), source, &mut nodes, None);
    Ok(nodes)
}

fn collect_nodes(
    ts_node: &tree_sitter::Node,
    source: &str,
    nodes: &mut Vec<RawNode>,
    parent_idx: Option<usize>,
) -> Option<usize> {
    let kind = ts_node.kind();
    let (node_kind, name) = match kind {
        "function_definition" => {
            let name = ts_node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|s| s.to_string());
            (NodeKind::Function, name)
        }
        "class_definition" => {
            let name = ts_node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|s| s.to_string());
            (NodeKind::Class, name)
        }
        "call" => {
            let name = ts_node
                .child_by_field_name("function")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|s| s.to_string());
            (NodeKind::CallSite, name)
        }
        "identifier" => {
            let name = ts_node
                .utf8_text(source.as_bytes())
                .ok()
                .map(|s| s.to_string());
            (NodeKind::Variable, name)
        }
        _ => {
            let mut latest_parent = parent_idx;
            for i in 0..ts_node.child_count() {
                let child = ts_node.child(i).unwrap();
                latest_parent = collect_nodes(&child, source, nodes, latest_parent);
            }
            return latest_parent;
        }
    };

    let start = ts_node.start_position();
    let end = ts_node.end_position();
    let idx = nodes.len();
    nodes.push(RawNode {
        language: Language::Python,
        kind: node_kind,
        name,
        usr: None,
        start_line: start.row + 1,
        start_col: start.column,
        end_line: end.row + 1,
        end_col: end.column,
        children: Vec::new(),
    });

    if let Some(pidx) = parent_idx {
        nodes[pidx].children.push(idx);
    }

    let new_parent = Some(idx);
    let mut current_parent = new_parent;
    for i in 0..ts_node.child_count() {
        let child = ts_node.child(i).unwrap();
        current_parent = collect_nodes(&child, source, nodes, current_parent);
    }
    new_parent
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
        let source = "def outer(): def inner(): pass";
        let facts = parse_python(source).expect("parsing should succeed");
        let outer = facts
            .iter()
            .find(|n| n.name.as_deref() == Some("outer"))
            .unwrap();
        let inner = facts
            .iter()
            .find(|n| n.name.as_deref() == Some("inner"))
            .unwrap();
        let outer_idx = facts
            .iter()
            .position(|n| n.name.as_deref() == Some("outer"))
            .unwrap();
        let inner_idx = facts
            .iter()
            .position(|n| n.name.as_deref() == Some("inner"))
            .unwrap();
        assert!(facts[outer_idx].children.contains(&inner_idx));
        // inner should not be a direct child of root
        assert_ne!(facts[outer_idx].kind, NodeKind::Module);
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
}
