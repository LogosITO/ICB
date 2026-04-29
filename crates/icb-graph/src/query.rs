use crate::graph::{CodePropertyGraph, Node};
use icb_common::NodeKind;

/// Returns all nodes of a given kind.
pub fn find_by_kind(cpg: &CodePropertyGraph, kind: NodeKind) -> Vec<&Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == kind)
        .collect()
}

/// Returns all call sites that target a function with the given name.
pub fn find_calls_to<'a>(cpg: &'a CodePropertyGraph, func_name: &str) -> Vec<&'a Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::CallSite && n.name.as_deref() == Some(func_name))
        .collect()
}
