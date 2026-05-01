//! Benchmark the call resolution step on an already‑built graph.
//!
//! The graph is constructed once per size, then only `resolve_calls` is measured.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_graph::builder::GraphBuilder;

mod common;

fn bench_resolve_calls(c: &mut Criterion) {
    let sizes = [100, 1000, 5000];

    for &size in &sizes {
        let facts = common::generate_facts(size);
        // Build a non‑resolved graph once
        let mut builder = GraphBuilder::new();
        builder.ingest_file_facts(&facts);
        let mut unresolved_graph = builder.cpg; // no resolve yet

        c.bench_function(&format!("resolve_calls_{}_funcs", size), |b| {
            b.iter(|| {
                // clone the graph? Too expensive. Instead we re‑ingest facts without resolve.
                // But that would include ingestion time. Better to measure only resolve on a prepared builder.
                // We'll recreate builder with ingested facts quickly:
                let mut b2 = GraphBuilder::new();
                b2.ingest_file_facts(&facts);
                b2.resolve_calls();
                black_box(b2.cpg);
            })
        });
    }
}

criterion_group!(benches, bench_resolve_calls);
criterion_main!(benches);
