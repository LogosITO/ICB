use crate::graph::{CodePropertyGraph, Edge};
use icb_common::NodeKind;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::fmt::Write;

/// Export the call graph to Graphviz DOT format.
///
/// Only function/class nodes and call edges are included.
pub fn export_call_dot(cpg: &CodePropertyGraph) -> String {
    let mut s = String::new();
    writeln!(s, "digraph CallGraph {{").unwrap();
    writeln!(s, "  rankdir=LR;").unwrap();
    writeln!(s, "  node [shape=box, style=rounded];").unwrap();

    for node_idx in cpg.graph.node_indices() {
        let node = &cpg.graph[node_idx];
        match node.kind {
            NodeKind::Function | NodeKind::Class => {
                let label = node.name.as_deref().unwrap_or("?");
                writeln!(
                    s,
                    "  n{} [label=\"{}\\nline {}\"];",
                    node_idx.index(),
                    label,
                    node.start_line
                )
                .unwrap();
            }
            _ => {}
        }
    }

    for edge_ref in cpg.graph.edge_references() {
        if *edge_ref.weight() == Edge::Call {
            writeln!(
                s,
                "  n{} -> n{};",
                edge_ref.source().index(),
                edge_ref.target().index()
            )
            .unwrap();
        }
    }

    writeln!(s, "}}").unwrap();
    s
}
