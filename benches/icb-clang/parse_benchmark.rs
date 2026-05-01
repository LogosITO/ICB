//! Performance benchmarks for the ICB Clang parser.
//!
//! # Usage
//!
//! ```bash
//! cargo bench -p icb-clang
//! ```
//!
//! HTML reports are generated in `target/criterion/`.
//!
//! # Scenarios
//!
//! 1. **single_large_file** – parse a synthetically generated source unit
//!    containing 1000 empty functions and 500 interleaved calls.  This
//!    stresses the visitor's ability to skip transparent AST nodes.
//!
//! 2. **real_project_directory** – recursively discover and parse all C/C++
//!    files under a real project (e.g. `../Vizora`).  This exercises the
//!    parallel directory walk and end‑to‑end fact extraction.
//!
//! The second benchmark is ignored by default (requires an external path).
//! Enable it by setting the environment variable `ICB_BENCH_PROJECT` to the
//! project root.

mod common;
mod single_large_file;
mod deeply_nested;
mod many_calls;
mod system_headers;
mod real_project_parallel;
mod real_project_sequential;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_clang::{parser, project};
use std::path::Path;

fn build_large_source(num_functions: usize) -> String {
    let mut src = String::with_capacity(num_functions * 64);
    for i in 0..num_functions {
        src.push_str(&format!("void func{}() {{}}\n", i));
        if i % 2 == 0 {
            src.push_str(&format!("void caller{}() {{ func{}(); }}\n", i, i));
        }
    }
    src
}

fn bench_single_large_file(c: &mut Criterion) {
    let source = build_large_source(1000);
    let args: Vec<String> = vec![];

    c.bench_function("parse_large_file_1000_funcs", |b| {
        b.iter(|| parser::parse_cpp_file(black_box(&source), black_box(&args), None, true).unwrap())
    });
}

fn bench_real_project(c: &mut Criterion) {
    let project_path =
        std::env::var("ICB_BENCH_PROJECT").unwrap_or_else(|_| "../Vizora".to_string());
    let root = Path::new(&project_path);
    if !root.exists() {
        eprintln!(
            "Skipping real project benchmark – path {:?} not found",
            root
        );
        return;
    }

    let args: Vec<String> = vec!["-std=c++17".into()];
    let allow_system = false;

    c.bench_function("parse_real_project_directory", |b| {
        b.iter(|| {
            project::parse_directory(
                black_box(root),
                black_box(&args),
                true, // parallel
                None, // no depth limit
                allow_system,
            )
            .unwrap()
        })
    });
}

criterion_group!(benches, bench_single_large_file, bench_real_project);
criterion_main!(benches);
