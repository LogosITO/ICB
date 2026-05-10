//! Benchmark: many function calls inside a single Rust source file.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use icb_rustc;
use std::fs;
use std::hint::black_box;
use tempfile::Builder;

mod common;

fn bench(c: &mut Criterion) {
    let call_counts = [1000, 10000, 50000];

    let args: Vec<String> = vec!["--edition".to_string(), "2021".to_string()];

    for &count in &call_counts {
        let source = common::build_many_calls_source(count);

        c.bench_function(&format!("rustc_many_calls_{}", count), |b| {
            b.iter_batched(
                || {
                    let tmp = Builder::new()
                        .prefix("icb_rustc_calls_")
                        .suffix(".rs")
                        .tempfile()
                        .unwrap();

                    fs::write(tmp.path(), &source).unwrap();

                    tmp
                },
                |tmp| {
                    let result =
                        icb_rustc::parse_rust_crate(black_box(tmp.path()), black_box(&args))
                            .unwrap();

                    black_box(result);
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
