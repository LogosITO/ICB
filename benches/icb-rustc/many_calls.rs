//! Benchmark: many function calls inside a single Rust source file.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_rustc;
use std::fs;
use tempfile::Builder;

mod common;

fn bench(c: &mut Criterion) {
    let call_counts = [1000, 5000, 20000];
    let args: Vec<String> = vec!["--edition".to_string(), "2021".to_string()];

    for &count in &call_counts {
        let source = common::build_many_calls_source(count);
        let tmp = Builder::new().suffix(".rs").tempfile().unwrap();
        fs::write(tmp.path(), &source).unwrap();

        c.bench_function(&format!("rustc_many_calls_{}", count), |b| {
            b.iter(|| icb_rustc::parse_rust_crate(black_box(tmp.path()), black_box(&args)).unwrap())
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
