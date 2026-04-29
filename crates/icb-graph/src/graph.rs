use icb_common::NodeKind;
use petgraph::stable_graph::StableGraph;

/// A node in the Code Property Graph.
///
/// Each node corresponds to a language entity (function, class, call,
/// etc.) and stores its kind, optional name, and source location.
#[derive(Debug, Clone)]
pub struct Node {
    /// Kind of the entity.
    pub kind: NodeKind,
    /// Human-readable name, if available.
    pub name: Option<String>,
    /// Unique symbol identifier (USR) used for deduplication across files.
    pub usr: Option<String>,
    /// Line where this entity starts (1-based).
    pub start_line: usize,
    /// Line where this entity ends (1-based).
    pub end_line: usize,
}

/// Edge types that can connect two nodes in the CPG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edge {
    /// AST parent-child relationship.
    AstChild,
    /// A call site that invokes a function/method.
    Call,
    /// A reference to a variable, class, etc.
    Reference,
}

/// Underlying graph type alias.
pub type CpgGraph = StableGraph<Node, Edge>;

/// Central container for the Code Property Graph.
///
/// Build instances using [`GraphBuilder`](super::builder::GraphBuilder) rather
/// than adding nodes manually.
#[derive(Default)]
pub struct CodePropertyGraph {
    /// The actual graph data structure.
    pub graph: CpgGraph,
}

impl CodePropertyGraph {
    /// Construct an empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Return the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
