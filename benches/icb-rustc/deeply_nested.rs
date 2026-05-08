//! Benchmark: deeply nested Rust AST (modules/structs).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_rustc;
use std::fs;
use tempfile::Builder;

mod common;

fn bench(c: &mut Criterion) {
    let depths = [5, 10, 20];
    let args: Vec<String> = vec!["--edition".to_string(), "2021".to_string()];

    for &depth in &depths {
        let source = common::build_deeply_nested_source(depth);
        let tmp = Builder::new().suffix(".rs").tempfile().unwrap();
        fs::write(tmp.path(), &source).unwrap();

        c.bench_function(&format!("rustc_deeply_nested_{}_levels", depth), |b| {
            b.iter(|| icb_rustc::parse_rust_crate(black_box(tmp.path()), black_box(&args)).unwrap())
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
