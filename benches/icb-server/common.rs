//! Shared helpers for server-side benchmarks.
//!
//! Provides a single function [`build_graph`] that constructs a
//! [`CodePropertyGraph`] of configurable size, complete with functions,
//! classes, methods and call edges, so that every benchmark works with
//! realistic data.

use icb_common::NodeKind;
use icb_graph::graph::{CodePropertyGraph, Edge, Node};

/// Build a synthetic graph containing `num_functions` function nodes,
/// some classes with methods, and random call edges.
///
/// The graph is designed to exercise the metrics computation and
/// subgraph filtering code paths.
pub fn build_graph(num_functions: usize) -> CodePropertyGraph {
    let mut cpg = CodePropertyGraph::new();
    let mut rng = fastrand::Rng::new();

    let mut func_indices = Vec::with_capacity(num_functions);

    for i in 0..num_functions {
        let idx = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some(format!("func_{}", i)),
            usr: Some(format!("file_{}.cpp", i % 10)),
            start_line: i * 2,
            end_line: i * 2 + 1,
        });
        func_indices.push(idx);
    }

    let num_classes = num_functions / 10;
    for c in 0..num_classes {
        let class_idx = cpg.graph.add_node(Node {
            kind: NodeKind::Class,
            name: Some(format!("Class_{}", c)),
            usr: Some(format!("file_{}.cpp", c % 5)),
            start_line: (c + 1) * 100,
            end_line: (c + 1) * 100 + 10,
        });
        for m in 0..3 {
            let method_idx = cpg.graph.add_node(Node {
                kind: NodeKind::Function,
                name: Some(format!("Class_{}_method_{}", c, m)),
                usr: Some(format!("file_{}.cpp", c % 5)),
                start_line: (c + 1) * 100 + 1 + m,
                end_line: (c + 1) * 100 + 2 + m,
            });
            cpg.graph.add_edge(class_idx, method_idx, Edge::AstChild);
            if !func_indices.is_empty() {
                let callee = rng.usize(0..func_indices.len());
                cpg.graph
                    .add_edge(method_idx, func_indices[callee], Edge::Call);
            }
        }
    }

    for _ in 0..num_functions * 2 {
        let caller = rng.usize(0..func_indices.len());
        let callee = rng.usize(0..func_indices.len());
        if caller != callee {
            cpg.graph
                .add_edge(func_indices[caller], func_indices[callee], Edge::Call);
        }
    }

    cpg
}
