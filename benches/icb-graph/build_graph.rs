//! Benchmark the full graph construction pipeline: from raw facts to a resolved CPG.
//!
//! Measures `GraphBuilder::ingest_file_facts` + `resolve_calls` for three sizes.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_graph::builder::GraphBuilder;

mod common;

fn bench_build_graph(c: &mut Criterion) {
    let sizes = [100, 1000, 5000];

    for &size in &sizes {
        let facts = common::generate_facts(size);
        c.bench_function(&format!("build_graph_{}_funcs", size), |b| {
            b.iter(|| {
                let mut builder = GraphBuilder::new();
                builder.ingest_file_facts(black_box(&facts));
                builder.resolve_calls();
                let _ = black_box(builder.cpg);
            })
        });
    }
}

criterion_group!(benches, bench_build_graph);
criterion_main!(benches);
