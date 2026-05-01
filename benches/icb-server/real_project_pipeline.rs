//! End‑to‑end pipeline benchmark on a real C/C++ project.
//!
//! Measures the complete server‑side workflow for a typical dashboard
//! request: parsing, graph construction with fact filtering, metric
//! computation, subgraph extraction, and JSON serialisation.
//!
//! The project path is taken from the environment variable
//! `ICB_BENCH_PROJECT`.  If the path does not exist the benchmark is
//! skipped.

use anyhow::Result;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icb_common::NodeKind;
use icb_graph::builder::GraphBuilder;
use icb_parser::facts::RawNode;
use icb_server::analytics;
use icb_server::display_name;
use icb_server::routes;

fn run_pipeline(project_path: &str) -> Result<()> {
    let root = std::path::Path::new(project_path);
    if !root.exists() {
        anyhow::bail!("project path does not exist");
    }

    let args: Vec<String> = vec!["-std=c++17".into()];
    let file_facts = icb_clang::project::parse_directory(root, &args, true, None, false)?;

    let mut builder = GraphBuilder::new();
    for (_, facts) in file_facts {
        let filtered: Vec<RawNode> = facts
            .into_iter()
            .filter(|f| {
                matches!(
                    f.kind,
                    NodeKind::Function | NodeKind::Class | NodeKind::CallSite
                )
            })
            .collect();
        let mut local = GraphBuilder::new();
        local.ingest_file_facts(&filtered);
        builder.merge(local);
    }
    builder.resolve_calls();
    let mut cpg = builder.cpg;
    display_name::cleanup_node_names(&mut cpg);

    let functions = analytics::collect_function_metrics(&cpg);
    let classes = analytics::collect_class_metrics(&cpg);
    let files = analytics::collect_file_metrics(&cpg);

    let subgraph = routes::__bench_focal_graph(&cpg, "main", 200, 2);

    let _ = black_box(serde_json::to_string(&functions)?);
    let _ = black_box(serde_json::to_string(&classes)?);
    let _ = black_box(serde_json::to_string(&files)?);
    let _ = black_box(serde_json::to_string(&subgraph)?);

    Ok(())
}

fn bench_real_pipeline(c: &mut Criterion) {
    let project_path = std::env::var("ICB_BENCH_PROJECT").unwrap_or_else(|_| "../Vizora".into());

    if !std::path::Path::new(&project_path).exists() {
        eprintln!("Skipping – path {:?} not found", project_path);
        return;
    }

    c.bench_function("real_project_pipeline", |b| {
        b.iter(|| run_pipeline(black_box(&project_path)).unwrap())
    });
}

criterion_group!(benches, bench_real_pipeline);
criterion_main!(benches);
