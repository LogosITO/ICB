//! Benchmark the call resolution step on an already‑built graph.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_graph::builder::GraphBuilder;

mod common;

fn bench_resolve_calls(c: &mut Criterion) {
    let sizes = [100, 1000, 5000];

    for &size in &sizes {
        let facts = common::generate_facts(size);

        c.bench_function(&format!("resolve_calls_{}_funcs", size), |b| {
            b.iter(|| {
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
