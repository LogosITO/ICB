use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_clang::project;
use std::path::Path;

fn bench(c: &mut Criterion) {
    let project_path = std::env::var("ICB_BENCH_PROJECT").unwrap_or_else(|_| "../Vizora".into());
    let root = Path::new(&project_path);
    if !root.exists() {
        eprintln!("Skipping – path {:?} not found", root);
        return;
    }
    let args: Vec<String> = vec!["-std=c++17".into()];
    c.bench_function("real_project_sequential", |b| {
        b.iter(|| {
            project::parse_directory(black_box(root), black_box(&args), false, None, false).unwrap()
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
