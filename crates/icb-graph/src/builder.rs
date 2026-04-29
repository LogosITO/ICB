use crate::graph::{CodePropertyGraph, Edge, Node};
use icb_parser::facts::RawNode;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;

/// Incrementally builds a [`CodePropertyGraph`] from parser facts.
///
/// `GraphBuilder` handles deduplication of symbols using the [`RawNode::usr`]
/// field (falling back to [`RawNode::name`]) and wires up AST parent-child
/// relationships.
///
/// # Example
///
/// ```rust
/// use icb_graph::builder::GraphBuilder;
/// use icb_parser::facts::RawNode;
/// use icb_common::{Language, NodeKind};
///
/// let mut builder = GraphBuilder::new();
/// let facts = vec![RawNode {
///     language: Language::Python,
///     kind: NodeKind::Function,
///     name: Some("main".into()),
///     usr: None,
///     start_line: 1,
///     start_col: 0,
///     end_line: 1,
///     end_col: 5,
///     children: vec![],
/// }];
/// builder.ingest_file_facts(&facts);
/// assert_eq!(builder.cpg.node_count(), 1);
/// ```
#[derive(Default)]
pub struct GraphBuilder {
    /// The Code Property Graph under construction.
    pub cpg: CodePropertyGraph,
    /// Map from symbol keys (USR or name) to graph node indices to avoid
    /// duplicates.
    symbol_index: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest facts from a single file.
    ///
    /// Each `RawNode` is converted to a graph node. If a node with the same
    /// unique key already exists (matching `usr` or `name`), the existing
    /// node is reused. AST parent-child relations are translated to
    /// [`Edge::AstChild`] edges.
    pub fn ingest_file_facts(&mut self, facts: &[RawNode]) {
        let mut map: HashMap<usize, NodeIndex> = HashMap::new();
        for (i, raw) in facts.iter().enumerate() {
            let usr: String = raw
                .usr
                .clone()
                .or_else(|| raw.name.clone())
                .unwrap_or_default();
            let node_idx = if let Some(&existing) = self.symbol_index.get(&usr) {
                existing
            } else {
                let idx = self.cpg.graph.add_node(Node {
                    kind: raw.kind.clone(),
                    name: raw.name.clone(),
                    usr: Some(usr.clone()),
                    start_line: raw.start_line,
                    end_line: raw.end_line,
                });
                self.symbol_index.insert(usr, idx);
                idx
            };
            map.insert(i, node_idx);
        }

        for (i, raw) in facts.iter().enumerate() {
            let from_idx = map[&i];
            for &child_raw_idx in &raw.children {
                if let Some(&to_idx) = map.get(&child_raw_idx) {
                    self.cpg.graph.add_edge(from_idx, to_idx, Edge::AstChild);
                }
            }
        }
    }

    /// Merge another builder into this one (reserved for parallel indexing).
    ///
    /// Currently not implemented. When parallel parsing is added, this
    /// method will combine two local graphs and unify their symbol indexes.
    pub fn merge(&mut self, _other: GraphBuilder) {
        unimplemented!("parallel merge not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icb_common::{Language, NodeKind};

    fn make_func_node(name: &str, line: usize) -> RawNode {
        RawNode {
            language: Language::Python,
            kind: NodeKind::Function,
            name: Some(name.into()),
            usr: None,
            start_line: line,
            start_col: 0,
            end_line: line,
            end_col: 10,
            children: vec![],
        }
    }

    #[test]
    fn test_deduplication_by_name() {
        let mut builder = GraphBuilder::new();
        let facts = vec![
            make_func_node("foo", 1),
            make_func_node("foo", 2), // duplicate
        ];
        builder.ingest_file_facts(&facts);
        assert_eq!(builder.cpg.node_count(), 1);
    }

    #[test]
    fn test_multiple_different_nodes() {
        let mut builder = GraphBuilder::new();
        let facts = vec![make_func_node("foo", 1), make_func_node("bar", 2)];
        builder.ingest_file_facts(&facts);
        assert_eq!(builder.cpg.node_count(), 2);
    }
}
