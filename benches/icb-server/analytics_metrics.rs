//! Benchmarks for the analytics module of `icb-server`.
//!
//! Measures the time to compute function, class, and file metrics on
//! graphs of increasing size.  These metrics are served by the
//! `/api/functions`, `/api/classes`, and `/api/files` endpoints.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_server::analytics;

mod common;

fn bench_analytics(c: &mut Criterion) {
    let sizes = [100, 500, 2000];

    for &size in &sizes {
        let graph = common::build_graph(size);

        c.bench_function(&format!("function_metrics_{}", size), |b| {
            b.iter(|| analytics::collect_function_metrics(black_box(&graph)))
        });

        c.bench_function(&format!("class_metrics_{}", size), |b| {
            b.iter(|| analytics::collect_class_metrics(black_box(&graph)))
        });

        c.bench_function(&format!("file_metrics_{}", size), |b| {
            b.iter(|| analytics::collect_file_metrics(black_box(&graph)))
        });
    }
}

criterion_group!(benches, bench_analytics);
criterion_main!(benches);
