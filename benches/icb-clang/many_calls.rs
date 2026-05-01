use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_clang::parser;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_many_calls_source(10000);
    let args: Vec<String> = vec![];
    c.bench_function("many_calls_10000", |b| {
        b.iter(|| parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
