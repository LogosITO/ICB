//! Benchmark tree‑sitter‑cpp, go, ruby on a single large file with 1000 functions.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_parser::cpp_tree_sitter;
use icb_parser::lang::go;
use icb_parser::lang::ruby;

mod common;

fn bench(c: &mut Criterion) {
    let cpp_src = common::build_large_flat_source(1000);
    let go_src = common::build_large_flat_go(1000);
    let ruby_src = common::build_large_flat_ruby(1000);

    c.bench_function("ts_cpp_large_file_1000_funcs", |b| {
        b.iter(|| cpp_tree_sitter::parse_cpp_file(black_box(&cpp_src)).unwrap())
    });

    c.bench_function("ts_go_large_file_1000_funcs", |b| {
        b.iter(|| go::parse_go_file(black_box(&go_src)).unwrap())
    });

    c.bench_function("ts_ruby_large_file_1000_funcs", |b| {
        b.iter(|| ruby::parse_ruby_file(black_box(&ruby_src)).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
