//! Benchmarks: impact of system header exclusion.
use criterion::{criterion_group, criterion_main, Criterion};
use icb_clang::parser;
use std::hint::black_box;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_source_with_system_include().to_string();
    let args: Vec<String> = vec![];

    c.bench_function("system_headers_on", |b| {
        b.iter(|| parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap())
    });
    c.bench_function("system_headers_off", |b| {
        b.iter(|| {
            parser::parse_cpp_file(black_box(&source), black_box(&args), None, false).unwrap()
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
