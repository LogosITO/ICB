use icb_common::NodeKind;
use petgraph::stable_graph::StableGraph;

/// A node in the Code Property Graph.
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub name: Option<String>,
    pub usr: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
}

/// Edge type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edge {
    AstChild,
    Call,
    Reference,
}

pub type CpgGraph = StableGraph<Node, Edge>;

/// Central Code Property Graph container.
#[derive(Default)]
pub struct CodePropertyGraph {
    pub graph: CpgGraph,
}

impl CodePropertyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
