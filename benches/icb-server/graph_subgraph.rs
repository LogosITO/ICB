//! Benchmarks for subgraph extraction routines.
//!
//! The `/api/graph` endpoint supports two query modes:
//!
//! * **focus** – expand from a given function name up to a certain depth.
//! * **kind filter** – return all nodes of a specific [`NodeKind`].
//!
//! Both modes are exercised here with realistic parameters.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_server::routes;

mod common;

fn bench_subgraphs(c: &mut Criterion) {
    let sizes = [100, 500, 2000];

    for &size in &sizes {
        let graph = common::build_graph(size);

        c.bench_function(&format!("focal_graph_depth2_{}", size), |b| {
            b.iter(|| routes::__bench_focal_graph(black_box(&graph), "func_0", 200, 2))
        });

        c.bench_function(&format!("subgraph_by_kind_function_{}", size), |b| {
            b.iter(|| routes::__bench_subgraph_by_kind(black_box(&graph), Some("Function"), 200))
        });
    }
}

criterion_group!(benches, bench_subgraphs);
criterion_main!(benches);
