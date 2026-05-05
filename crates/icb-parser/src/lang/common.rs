//! Common tree-sitter traversal utilities shared across language parsers.
//!
//! Provides functions to create [`RawNode`] entries and to recursively walk
//! a [`tree_sitter::Tree`] with a language-specific node classifier.  Using
//! this module avoids duplicating the traversal logic for each supported
//! language.

use crate::facts::RawNode;
use icb_common::{Language, NodeKind};
use tree_sitter::Node;

/// Create a [`RawNode`] from the supplied metadata and append it to `facts`.
///
/// Returns the index of the newly inserted node so that parent/child
/// relationships can be maintained.
pub fn push_node(
    facts: &mut Vec<RawNode>,
    language: Language,
    kind: NodeKind,
    name: Option<String>,
    node: &Node,
    parent_idx: Option<usize>,
) -> usize {
    let start = node.start_position();
    let end = node.end_position();
    let start_line = start.row + 1;
    let end_line = std::cmp::max(end.row + 1, start_line);

    let idx = facts.len();
    facts.push(RawNode {
        language,
        kind,
        name,
        usr: None,
        start_line,
        start_col: start.column,
        end_line,
        end_col: end.column,
        children: Vec::new(),
        source_file: None,
    });

    if let Some(pidx) = parent_idx {
        facts[pidx].children.push(idx);
    }
    idx
}

/// Find the first direct child of `node` whose `kind()` equals `kind`.
///
/// The search stops at the first match and does **not** allocate an
/// intermediate collection.
pub fn child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let mut children = node.children(&mut cursor);
    children.find(|c| c.kind() == kind)
}

/// Generic recursive traversal of a tree-sitter sub‑tree.
///
/// For every node the `classifier` closure is called.  If it returns
/// `Some((node_kind, name, is_container))` a [`RawNode`] is recorded and
/// – when `is_container` is `true` – its children are traversed with the
/// node itself becoming the new parent.  If `classifier` returns `None`
/// the children are still visited, but the current parent remains
/// unchanged.
pub fn traverse_node<F>(
    node: Node,
    source: &str,
    facts: &mut Vec<RawNode>,
    parent_idx: Option<usize>,
    language: Language,
    classifier: &F,
) -> Option<usize>
where
    F: Fn(&Node, &str) -> Option<(NodeKind, Option<String>, bool)>,
{
    if let Some((kind, name, is_container)) = classifier(&node, source) {
        let idx = push_node(facts, language, kind, name, &node, parent_idx);
        if is_container {
            let new_parent = Some(idx);
            let mut current_parent = new_parent;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                current_parent =
                    traverse_node(child, source, facts, current_parent, language, classifier);
            }
            new_parent
        } else {
            parent_idx
        }
    } else {
        let mut current_parent = parent_idx;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            current_parent =
                traverse_node(child, source, facts, current_parent, language, classifier);
        }
        current_parent
    }
}
