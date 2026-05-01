//! Benchmark tree-sitter-cpp on a file with 10000 call expressions.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_parser::cpp_tree_sitter;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_many_calls_source(10000);
    c.bench_function("ts_cpp_many_calls_10000", |b| {
        b.iter(|| cpp_tree_sitter::parse_cpp_file(black_box(&source)).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
