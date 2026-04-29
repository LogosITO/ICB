use crate::graph::{CodePropertyGraph, Edge, Node};
use icb_parser::facts::RawNode;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;

#[derive(Default)]
pub struct GraphBuilder {
    pub cpg: CodePropertyGraph,
    symbol_index: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest facts from a single file into this builder.
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

    /// Merge another builder into this one (for parallel workflows).
    pub fn merge(&mut self, _other: GraphBuilder) {
        // TODO: implement full merging of nodes/edges
        unimplemented!()
    }
}
