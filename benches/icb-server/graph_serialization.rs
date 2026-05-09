use criterion::{criterion_group, criterion_main, Criterion};
use icb_common::NodeKind;
use icb_graph::graph::{CodePropertyGraph, GraphData};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::hint::black_box;

fn make_test_graph(size: usize) -> CodePropertyGraph {
    let mut cpg = CodePropertyGraph::new();
    for i in 0..size {
        cpg.graph.add_node(icb_graph::graph::Node {
            kind: NodeKind::Function,
            name: Some(format!("func{}", i)),
            usr: Some("bench.cpp".into()),
            start_line: i + 1,
            end_line: i + 1,
        });
    }
    cpg
}

fn bench(c: &mut Criterion) {
    for &size in &[100, 500, 2000] {
        let graph = make_test_graph(size);
        let data = {
            let nodes: Vec<_> = graph.graph.node_weights().cloned().collect();
            let edges: Vec<_> = graph
                .graph
                .edge_references()
                .map(|e| (e.source().index(), e.target().index(), e.weight().clone()))
                .collect();
            GraphData { nodes, edges }
        };
        c.bench_function(&format!("json_serialize_{}_nodes", size), |b| {
            b.iter(|| serde_json::to_string(black_box(&data)).unwrap())
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
