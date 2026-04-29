use crate::facts::RawNode;
use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::Parser;

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
