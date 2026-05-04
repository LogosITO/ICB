//! Go language parser using tree-sitter-go.
//!
//! Extracts function declarations, method declarations, call expressions,
//! and type declarations (struct/interface) from Go source files.

use icb_common::{IcbError, Language, NodeKind};
use tree_sitter::{Node, Parser};

use crate::facts::RawNode;

pub fn parse_go_file(source: &str) -> Result<Vec<RawNode>, IcbError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::language())
        .map_err(|e| IcbError::Parse(format!("cannot set tree-sitter-go language: {e}")))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| IcbError::Parse("tree-sitter parse returned None".into()))?;

    let mut facts = Vec::new();
    traverse_node(tree.root_node(), source, &mut facts, None);
    Ok(facts)
}

fn traverse_node(
    node: Node,
    source: &str,
    facts: &mut Vec<RawNode>,
    parent_idx: Option<usize>,
) -> Option<usize> {
    let kind = node.kind();

    let (node_kind, name, is_container) = match kind {
        "function_declaration" | "method_declaration" => {
            let name = child_of_kind(node, "identifier")
                .map(|n| {
                    n.utf8_text(source.as_bytes())
                        .unwrap_or_default()
                        .to_string()
                })
                .unwrap_or_default();
            (NodeKind::Function, Some(name), true)
        }
        "type_declaration" => {
            if let Some(type_spec) = child_of_kind(node, "type_spec") {
                if child_of_kind(type_spec, "struct_type").is_some()
                    || child_of_kind(type_spec, "interface_type").is_some()
                {
                    let name = child_of_kind(type_spec, "identifier")
                        .map(|n| {
                            n.utf8_text(source.as_bytes())
                                .unwrap_or_default()
                                .to_string()
                        })
                        .unwrap_or_default();
                    return Some(create_node(
                        facts,
                        NodeKind::Class,
                        Some(name),
                        &node,
                        parent_idx,
                    ));
                }
            }
            let mut current_parent = parent_idx;
            for child in node.children(&mut node.walk()) {
                current_parent = traverse_node(child, source, facts, current_parent);
            }
            return current_parent;
        }
        "call_expression" => {
            let name_node = child_of_kind(node, "identifier")
                .or_else(|| child_of_kind(node, "selector_expression"));
            let name = name_node
                .map(|n| {
                    n.utf8_text(source.as_bytes())
                        .unwrap_or_default()
                        .to_string()
                })
                .unwrap_or_default();
            (NodeKind::CallSite, Some(name), false)
        }
        _ => {
            let mut current_parent = parent_idx;
            for child in node.children(&mut node.walk()) {
                current_parent = traverse_node(child, source, facts, current_parent);
            }
            return current_parent;
        }
    };

    let idx = create_node(facts, node_kind, name, &node, parent_idx);
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

fn create_node(
    facts: &mut Vec<RawNode>,
    kind: NodeKind,
    name: Option<String>,
    node: &Node,
    parent_idx: Option<usize>,
) -> usize {
    let start = node.start_position();
    let end = node.end_position();

    let idx = facts.len();
    facts.push(RawNode {
        language: Language::Go,
        kind,
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
    idx
}

fn child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    children.into_iter().find(|child| child.kind() == kind)
}
