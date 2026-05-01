//! Benchmark tree-sitter-cpp on a real C/C++ project.
//!
//! Reads files from `ICB_BENCH_PROJECT` environment variable, parses each
//! file with tree‑sitter, and sums the number of facts.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_common::Language;
use icb_parser::manager::ParserManager;
use std::path::Path;

fn bench(c: &mut Criterion) {
    let project_path = std::env::var("ICB_BENCH_PROJECT").unwrap_or_else(|_| "../Vizora".into());
    let root = Path::new(&project_path);
    if !root.exists() {
        eprintln!("Skipping – path {:?} not found", root);
        return;
    }

    let manager = ParserManager::new();
    c.bench_function("ts_cpp_real_project", |b| {
        b.iter(|| {
            manager
                .parse_directory(Language::CppTreeSitter, black_box(root))
                .unwrap()
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
