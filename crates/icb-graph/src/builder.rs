use crate::graph::{CodePropertyGraph, Edge, Node};
use icb_common::NodeKind;
use icb_parser::facts::RawNode;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::HashMap;

/// Incrementally builds a [`CodePropertyGraph`] from parser facts.
///
/// Supports ingesting facts from multiple files and merging local graphs
/// (e.g., from parallel parsing) into a single global graph with symbol
/// deduplication. Call [`Self::resolve_calls`] after all facts are ingested
/// to wire up call sites to their definitions.
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
///     source_file: None,
/// }];
/// builder.ingest_file_facts(&facts);
/// builder.resolve_calls();
/// assert_eq!(builder.cpg.node_count(), 1);
/// ```
#[derive(Default)]
pub struct GraphBuilder {
    pub cpg: CodePropertyGraph,
    symbol_index: HashMap<String, NodeIndex>,
    function_defs: HashMap<String, Vec<NodeIndex>>,
    call_sites: HashMap<String, Vec<NodeIndex>>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest facts from a single file.
    ///
    /// This method may be called multiple times (even from different threads
    /// after merging). Each call enriches the same graph.
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

            if let Some(name) = &raw.name {
                match raw.kind {
                    NodeKind::Function | NodeKind::Class => {
                        self.function_defs
                            .entry(name.clone())
                            .or_default()
                            .push(node_idx);
                    }
                    NodeKind::CallSite => {
                        self.call_sites
                            .entry(name.clone())
                            .or_default()
                            .push(node_idx);
                    }
                    _ => {}
                }
            }
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

    /// Resolve calls: for every call site with a matching function/class
    /// definition, add a [`Edge::Call`] edge from the call site to the definition.
    pub fn resolve_calls(&mut self) {
        for (name, call_indices) in &self.call_sites {
            if let Some(def_indices) = self.function_defs.get(name) {
                for &call_idx in call_indices {
                    for &def_idx in def_indices {
                        self.cpg.graph.add_edge(call_idx, def_idx, Edge::Call);
                    }
                }
            }
        }
    }

    /// Merge another `GraphBuilder` into this one.
    ///
    /// All nodes from `other` are transferred to `self`. Nodes with the
    /// same symbolic key (USR or name) that already exist in `self` are
    /// reused, and edges are rewired accordingly. The temporary call/def
    /// maps are also merged.
    pub fn merge(&mut self, other: GraphBuilder) {
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
        for edge_ref in other.cpg.graph.edge_references() {
            let src = edge_ref.source();
            let tgt = edge_ref.target();
            if let (Some(&new_src), Some(&new_tgt)) = (node_map.get(&src), node_map.get(&tgt)) {
                self.cpg
                    .graph
                    .add_edge(new_src, new_tgt, edge_ref.weight().clone());
            }
        }
        for (name, defs) in other.function_defs {
            let entry = self.function_defs.entry(name).or_default();
            for idx in defs {
                if let Some(&new_idx) = node_map.get(&idx) {
                    entry.push(new_idx);
                }
            }
        }
        for (name, calls) in other.call_sites {
            let entry = self.call_sites.entry(name).or_default();
            for idx in calls {
                if let Some(&new_idx) = node_map.get(&idx) {
                    entry.push(new_idx);
                }
            }
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
            source_file: None,
        }
    }

    fn make_call_node(name: &str, line: usize) -> RawNode {
        RawNode {
            language: Language::Python,
            kind: NodeKind::CallSite,
            name: Some(name.into()),
            usr: None,
            start_line: line,
            start_col: 0,
            end_line: line,
            end_col: 5,
            children: vec![],
            source_file: None,
        }
    }

    #[test]
    fn test_deduplication_by_name() {
        let mut builder = GraphBuilder::new();
        let facts = vec![make_func_node("foo", 1), make_func_node("foo", 2)];
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
        b2.ingest_file_facts(&[make_func_node("bar", 2), make_func_node("foo", 3)]);
        b1.merge(b2);
        assert_eq!(b1.cpg.node_count(), 2);
    }

    #[test]
    fn test_resolve_calls_creates_edges() {
        let mut builder = GraphBuilder::new();
        let facts = vec![make_func_node("foo", 1), make_call_node("foo", 2)];
        builder.ingest_file_facts(&facts);
        builder.resolve_calls();
        let call_edges: Vec<_> = builder
            .cpg
            .graph
            .edge_references()
            .filter(|e| *e.weight() == Edge::Call)
            .collect();
        assert!(!call_edges.is_empty());
    }
}
