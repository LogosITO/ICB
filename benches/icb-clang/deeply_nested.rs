use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_clang::parser;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_deeply_nested_source(5);
    let args: Vec<String> = vec![];
    c.bench_function("deeply_nested_5_levels", |b| {
        b.iter(|| parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
