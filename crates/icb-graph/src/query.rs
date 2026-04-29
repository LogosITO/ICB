use crate::graph::{CodePropertyGraph, Node};
use icb_common::NodeKind;

/// Return all nodes of the given `kind`.
///
/// # Examples
///
/// ```rust
/// use icb_graph::graph::CodePropertyGraph;
/// use icb_graph::query::find_by_kind;
/// use icb_common::NodeKind;
///
/// let mut cpg = CodePropertyGraph::new();
/// // populate cpg...
/// let functions = find_by_kind(&cpg, NodeKind::Function);
/// ```
pub fn find_by_kind(cpg: &CodePropertyGraph, kind: NodeKind) -> Vec<&Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == kind)
        .collect()
}

/// Return all call sites that target a function with the given `func_name`.
///
/// The comparison is done only on the node's name; no resolution through
/// scopes is performed yet.
///
/// # Examples
///
/// ```rust
/// use icb_graph::graph::CodePropertyGraph;
/// use icb_graph::query::find_calls_to;
///
/// let cpg = CodePropertyGraph::new();
/// let calls = find_calls_to(&cpg, "println");
/// assert!(calls.is_empty());
/// ```
pub fn find_calls_to<'a>(cpg: &'a CodePropertyGraph, func_name: &str) -> Vec<&'a Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::CallSite && n.name.as_deref() == Some(func_name))
        .collect()
}
