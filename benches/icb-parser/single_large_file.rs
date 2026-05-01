//! Benchmark tree-sitter-cpp on a single large file with 1000 functions.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_parser::cpp_tree_sitter;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_large_flat_source(1000);
    c.bench_function("ts_cpp_large_file_1000_funcs", |b| {
        b.iter(|| cpp_tree_sitter::parse_cpp_file(black_box(&source)).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
