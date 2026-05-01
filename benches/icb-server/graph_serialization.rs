//! Benchmarks for JSON serialization of the graph data structure.
//!
//! The server returns a [`GraphData`] object for every `/api/graph` request.
//! This benchmark measures the cost of converting that structure into a
//! JSON string, which is the dominant factor in response time for large
//! subgraphs.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_graph::graph::Edge;
use icb_graph::graph::GraphData;

mod common;

fn bench_serialization(c: &mut Criterion) {
    let sizes = [100, 500, 2000];

    for &size in &sizes {
        let graph = common::build_graph(size);

        let data = GraphData {
            nodes: graph.graph.node_weights().cloned().collect(),
            edges: graph
                .graph
                .edge_indices()
                .map(|e| {
                    let (src, tgt) = graph.graph.edge_endpoints(e).unwrap();
                    (src.index(), tgt.index(), graph.graph[e].clone())
                })
                .collect(),
        };

        c.bench_function(&format!("graph_json_serialize_{}", size), |b| {
            b.iter(|| serde_json::to_string(black_box(&data)).unwrap())
        });
    }
}

criterion_group!(benches, bench_serialization);
criterion_main!(benches);
