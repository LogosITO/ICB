//! Benchmark tree‑sitter‑cpp, go, ruby on deeply nested structs/modules.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_parser::cpp_tree_sitter;
use icb_parser::lang::go;
use icb_parser::lang::ruby;

mod common;

fn bench(c: &mut Criterion) {
    let cpp_src = common::build_deeply_nested_source(5);
    let go_src = common::build_deeply_nested_go(5);
    let ruby_src = common::build_deeply_nested_ruby(5);

    c.bench_function("ts_cpp_deeply_nested_5_levels", |b| {
        b.iter(|| cpp_tree_sitter::parse_cpp_file(black_box(&cpp_src)).unwrap())
    });

    c.bench_function("ts_go_deeply_nested_5_levels", |b| {
        b.iter(|| go::parse_go_file(black_box(&go_src)).unwrap())
    });

    c.bench_function("ts_ruby_deeply_nested_5_levels", |b| {
        b.iter(|| ruby::parse_ruby_file(black_box(&ruby_src)).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
