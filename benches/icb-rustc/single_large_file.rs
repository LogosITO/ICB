//! Benchmark: parsing a single large Rust source file with varying function counts.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use icb_rustc;
use std::fs;
use std::hint::black_box;
use tempfile::Builder;

mod common;

fn bench(c: &mut Criterion) {
    let sizes = [100, 1000, 5000];

    let args: Vec<String> = vec!["--edition".to_string(), "2021".to_string()];

    for &size in &sizes {
        let source = common::build_large_flat_source(size);

        c.bench_function(&format!("rustc_single_large_file_{}_funcs", size), |b| {
            b.iter_batched(
                || {
                    let tmp = Builder::new()
                        .prefix("icb_rustc_large_")
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
