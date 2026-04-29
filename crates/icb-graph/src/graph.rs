use icb_common::NodeKind;
use petgraph::stable_graph::StableGraph;
use serde::{Deserialize, Serialize};

/// A node in the Code Property Graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub kind: NodeKind,
    pub name: Option<String>,
    pub usr: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
}

/// Edge type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edge {
    AstChild,
    Call,
    Reference,
}

pub type CpgGraph = StableGraph<Node, Edge>;

/// Central Code Property Graph container.
#[derive(Debug)]
pub struct CodePropertyGraph {
    pub graph: CpgGraph,
}

impl CodePropertyGraph {
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

impl Default for CodePropertyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Intermediate representation for (de)serialization.
#[derive(Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<(usize, usize, Edge)>,
}

impl From<&CodePropertyGraph> for GraphData {
    fn from(cpg: &CodePropertyGraph) -> Self {
        let nodes: Vec<Node> = cpg.graph.node_weights().cloned().collect();
        let mut edges = Vec::new();
        for edge_idx in cpg.graph.edge_indices() {
            if let Some((src, tgt)) = cpg.graph.edge_endpoints(edge_idx) {
                let weight = cpg.graph[edge_idx].clone();
                edges.push((src.index(), tgt.index(), weight));
            }
        }
        GraphData { nodes, edges }
    }
}

impl From<GraphData> for CodePropertyGraph {
    fn from(data: GraphData) -> Self {
        let mut graph: CpgGraph = StableGraph::new();
        let mut indices: Vec<petgraph::stable_graph::NodeIndex> =
            Vec::with_capacity(data.nodes.len());
        for node in data.nodes {
            let idx = graph.add_node(node);
            indices.push(idx);
        }
        for (src, tgt, edge) in data.edges {
            if src < indices.len() && tgt < indices.len() {
                graph.add_edge(indices[src], indices[tgt], edge);
            }
        }
        CodePropertyGraph { graph }
    }
}
