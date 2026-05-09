//! Benchmarks: many function calls inside a single translation unit.
use criterion::{criterion_group, criterion_main, Criterion};
use icb_clang::parser;
use std::hint::black_box;

mod common;

fn bench(c: &mut Criterion) {
    let args: Vec<String> = vec![];
    for &count in &[1000, 5000, 20000] {
        let source = common::build_many_calls_source(count);
        c.bench_function(&format!("many_calls_{}", count), |b| {
            b.iter(|| {
                parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap()
            })
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
