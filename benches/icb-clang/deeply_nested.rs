//! Benchmarks: deeply nested AST (classes/namespaces).
use criterion::{criterion_group, criterion_main, Criterion};
use icb_clang::parser;
use std::hint::black_box;

mod common;

fn bench(c: &mut Criterion) {
    let args: Vec<String> = vec![];
    for &depth in &[5, 10, 20, 50] {
        let source = common::build_deeply_nested_source(depth);
        c.bench_function(&format!("deeply_nested_{}_levels", depth), |b| {
            b.iter(|| {
                parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap()
            })
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
