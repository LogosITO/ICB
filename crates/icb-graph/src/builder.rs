use crate::graph::{CodePropertyGraph, Edge, Node};
use icb_parser::facts::RawNode;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;

/// Incrementally builds a [`CodePropertyGraph`] from parser facts.
///
/// Supports ingesting facts from multiple files and merging local graphs
/// (e.g., from parallel parsing) into a single global graph with symbol
/// deduplication.
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
    /// Map from symbolic keys (USR or name) to graph node indices for
    /// deduplication.
    symbol_index: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest facts from a single file.
    ///
    /// This method may be called multiple times (even from different threads
    /// after merging) — each call enriches the same graph.
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

    /// Merge another `GraphBuilder` into this one.
    ///
    /// All nodes from `other` are transferred to `self`. Nodes with the
    /// same symbolic key (USR or name) that already exist in `self` are
    /// reused, and edges are rewired accordingly. This is the main
    /// mechanism to combine graphs from parallel file parsing.
    pub fn merge(&mut self, other: GraphBuilder) {
        // First, transfer all nodes from other, reusing existing ones
        let mut node_map: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        for old_idx in other.cpg.graph.node_indices() {
            let node = &other.cpg.graph[old_idx];
            let usr = node.usr.clone().unwrap_or_default();
            let new_idx = if let Some(&existing) = self.symbol_index.get(&usr) {
                existing
            } else {
                let idx = self.cpg.graph.add_node(node.clone());
                self.symbol_index.insert(usr, idx);
                idx
            };
            node_map.insert(old_idx, new_idx);
        }
        // Transfer edges
        for edge_ref in other.cpg.graph.edge_indices() {
            let (source, target) = other.cpg.graph.edge_endpoints(edge_ref).unwrap();
            let new_source = node_map[&source];
            let new_target = node_map[&target];
            self.cpg
                .graph
                .add_edge(new_source, new_target, other.cpg.graph[edge_ref].clone());
        }
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

    #[test]
    fn test_merge_two_builders() {
        let mut b1 = GraphBuilder::new();
        b1.ingest_file_facts(&[make_func_node("foo", 1)]);
        let mut b2 = GraphBuilder::new();
        b2.ingest_file_facts(&[make_func_node("bar", 2), make_func_node("foo", 3)]); // duplicate foo
        b1.merge(b2);
        assert_eq!(b1.cpg.node_count(), 2); // foo, bar
    }
}
