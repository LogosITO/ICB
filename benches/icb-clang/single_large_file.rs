//! Benchmarks: parsing a single large source file with varying function counts.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_clang::parser;

mod common;

fn bench(c: &mut Criterion) {
    let args: Vec<String> = vec![];
    for &n in &[100, 500, 2000, 5000] {
        let source = common::build_large_flat_source(n);
        c.bench_function(&format!("single_large_file_{}_funcs", n), |b| {
            b.iter(|| {
                parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap()
            })
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
