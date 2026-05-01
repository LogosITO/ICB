use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_clang::parser;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_large_flat_source(1000);
    let args: Vec<String> = vec![];
    c.bench_function("single_large_file_1000_funcs", |b| {
        b.iter(|| parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
