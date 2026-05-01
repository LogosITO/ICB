//! Benchmark tree-sitter-cpp on deeply nested structs.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_parser::cpp_tree_sitter;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_deeply_nested_source(5);
    c.bench_function("ts_cpp_deeply_nested_5_levels", |b| {
        b.iter(|| cpp_tree_sitter::parse_cpp_file(black_box(&source)).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
