use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_clang::parser;

mod common;

fn bench(c: &mut Criterion) {
    let source = common::build_source_with_system_include().to_string();
    let args: Vec<String> = vec![];

    c.bench_function("with_system_headers", |b| {
        b.iter(|| parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap())
    });

    c.bench_function("without_system_headers", |b| {
        b.iter(|| {
            parser::parse_cpp_file(black_box(&source), black_box(&args), None, false).unwrap()
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
