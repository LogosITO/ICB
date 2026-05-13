use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct Output {
    metadata: Metadata,
    crates: BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>>,
}

#[derive(Serialize)]
struct Metadata {
    date: String,
    commit: String,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        args[1].clone()
    } else {
        "target/criterion".into()
    };
    let mut crates: BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>> = BTreeMap::new();
    let path = Path::new(&input);
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let bench_dir = entry.path();
            if !bench_dir.is_dir() {
                continue;
            }
            let bench_name = entry.file_name().to_string_lossy().to_string();
            let estimates = bench_dir.join("new").join("estimates.json");
            if !estimates.exists() {
                continue;
            }
            let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&estimates)?)?;
            let point_estimate = json["mean"]["point_estimate"].as_f64().unwrap_or(0.0);
            let (crate_name, scenario, backend) = classify(&bench_name);
            crates
                .entry(crate_name)
                .or_default()
                .entry(scenario)
                .or_default()
                .insert(backend, point_estimate);
        }
    }
    let output = Output {
        metadata: Metadata {
            date: chrono::Utc::now().to_rfc3339(),
            commit: env::var("GITHUB_SHA").unwrap_or_default(),
        },
        crates,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn classify(bench_name: &str) -> (String, String, String) {
    let name = bench_name;
    if name.starts_with("build_graph_") {
        return ("icb-graph".into(), "Build Graph".into(), "rust".into());
    }
    if name.starts_with("full_analysis_") {
        return ("icb-graph".into(), "Full Analysis".into(), "rust".into());
    }
    if name.starts_with("resolve_calls_") {
        return ("icb-graph".into(), "Resolve Calls".into(), "rust".into());
    }
    if name.starts_with("focal_graph_depth2_") {
        return (
            "icb-graph".into(),
            "Focal Graph Depth2".into(),
            "rust".into(),
        );
    }
    if name.starts_with("class_metrics_") {
        return ("icb-server".into(), "Class Metrics".into(), "rust".into());
    }
    if name.starts_with("file_metrics_") {
        return ("icb-server".into(), "File Metrics".into(), "rust".into());
    }
    if name.starts_with("function_metrics_") {
        return (
            "icb-server".into(),
            "Function Metrics".into(),
            "rust".into(),
        );
    }
    if name.starts_with("graph_json_serialize_") {
        return (
            "icb-server".into(),
            "Graph Json Serialize".into(),
            "rust".into(),
        );
    }
    if name.starts_with("subgraph_by_kind_function_") {
        return (
            "icb-server".into(),
            "Subgraph By Kind Function".into(),
            "rust".into(),
        );
    }
    if name == "real_project_pipeline" {
        return (
            "icb-server".into(),
            "Real Project Pipeline".into(),
            "rust".into(),
        );
    }
    if name.starts_with("deeply_nested_") {
        return ("icb-clang".into(), "Deeply Nested".into(), "clang".into());
    }
    if name.starts_with("many_calls_") {
        return ("icb-clang".into(), "Many Calls".into(), "clang".into());
    }
    if name.starts_with("single_large_file_") {
        return (
            "icb-clang".into(),
            "Single Large File".into(),
            "clang".into(),
        );
    }
    if name.starts_with("real_project_parallel") {
        return (
            "icb-clang".into(),
            "Real Project Parallel".into(),
            "clang".into(),
        );
    }
    if name.starts_with("real_project_sequential") {
        return (
            "icb-clang".into(),
            "Real Project Sequential".into(),
            "clang".into(),
        );
    }
    if name.starts_with("with_system_headers") {
        return (
            "icb-clang".into(),
            "With System Headers".into(),
            "clang".into(),
        );
    }
    if name.starts_with("without_system_headers") {
        return (
            "icb-clang".into(),
            "Without System Headers".into(),
            "clang".into(),
        );
    }
    if let Some(rest) = name.strip_prefix("ts_") {
        let mut parts = rest.splitn(2, '_');
        let lang = parts.next().unwrap_or("unknown");
        let scenario_raw = parts.next().unwrap_or("unknown");
        let scenario = scenario_raw
            .split('_')
            .take_while(|s| !s.chars().all(char::is_numeric))
            .collect::<Vec<_>>()
            .join("_");
        let scenario_name = match scenario.as_str() {
            "deeply_nested" => "Deeply Nested",
            "many_calls" => "Many Calls",
            "single_large_file" => "Single Large File",
            "real_project" => "Real Project",
            _ => scenario_raw,
        };
        let backend = format!("ts_{}", lang);
        return ("icb-parser".into(), scenario_name.into(), backend);
    }
    if name == "report" {
        return ("icb-report".into(), "Report".into(), "rust".into());
    }
    ("unknown".into(), name.into(), "unknown".into())
}
