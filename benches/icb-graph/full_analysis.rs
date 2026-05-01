//! Benchmark the complete analysis suite on a resolved graph.
//!
//! Includes cycle detection, dead code detection, and complexity computation.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_graph::analysis;

mod common;

fn bench_full_analysis(c: &mut Criterion) {
    let sizes = [100, 1000, 5000];

    for &size in &sizes {
        let graph = common::build_graph(size);
        c.bench_function(&format!("full_analysis_{}_funcs", size), |b| {
            b.iter(|| {
                analysis::detect_call_cycles(black_box(&graph));
                analysis::detect_dead_code(black_box(&graph), &["main".to_string()]);
                analysis::detect_complex_functions(black_box(&graph), 0);
            })
        });
    }
}

criterion_group!(benches, bench_full_analysis);
criterion_main!(benches);
