//! Benchmarks: parallel parsing of a real project (set via ICB_BENCH_PROJECT env).
use criterion::{criterion_group, criterion_main, Criterion};
use icb_clang::project;
use std::hint::black_box;
use std::path::Path;

fn bench(c: &mut Criterion) {
    let root = std::env::var("ICB_BENCH_PROJECT")
        .ok()
        .and_then(|p| {
            let path = Path::new(&p);
            if path.exists() {
                Some(p)
            } else {
                None
            }
        })
        .unwrap_or_else(|| "../Vizora".into());
    let args: Vec<String> = vec!["-std=c++17".into()];
    c.bench_function("real_project_parallel", |b| {
        b.iter(|| {
            project::parse_directory(
                black_box(Path::new(&root)),
                black_box(&args),
                true,
                None,
                false,
            )
            .unwrap()
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
