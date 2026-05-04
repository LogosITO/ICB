//! Benchmark tree‑sitter‑cpp, go, ruby on a file with 10 000 call expressions.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_parser::cpp_tree_sitter;
use icb_parser::lang::go;
use icb_parser::lang::ruby;

mod common;

fn bench(c: &mut Criterion) {
    let cpp_src = common::build_many_calls_source(10_000);
    let go_src = common::build_many_calls_go(10_000);
    let ruby_src = common::build_many_calls_ruby(10_000);

    c.bench_function("ts_cpp_many_calls_10000", |b| {
        b.iter(|| cpp_tree_sitter::parse_cpp_file(black_box(&cpp_src)).unwrap())
    });

    c.bench_function("ts_go_many_calls_10000", |b| {
        b.iter(|| go::parse_go_file(black_box(&go_src)).unwrap())
    });

    c.bench_function("ts_ruby_many_calls_10000", |b| {
        b.iter(|| ruby::parse_ruby_file(black_box(&ruby_src)).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
