use anyhow::{Context, Result};
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
        parse_criterion_dir(path, &mut crates)?;
    } else {
        parse_bench_txt(path, &mut crates)?;
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

fn parse_criterion_dir(
    root: &Path,
    crates: &mut BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>>,
) -> Result<()> {
    for group_entry in fs::read_dir(root)? {
        let group_entry = group_entry?;
        let group_path = group_entry.path();
        if !group_path.is_dir() {
            continue;
        }
        let group_name = group_entry.file_name().to_string_lossy().to_string();

        for bench_entry in fs::read_dir(&group_path)? {
            let bench_entry = bench_entry?;
            let bench_path = bench_entry.path();
            if !bench_path.is_dir() {
                continue;
            }
            let bench_name = bench_entry.file_name().to_string_lossy().to_string();

            let estimates = bench_path.join("new").join("estimates.json");
            if !estimates.exists() {
                continue;
            }

            let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&estimates)?)?;
            let point_estimate = json["mean"]["point_estimate"].as_f64().unwrap_or(0.0);

            let (crate_name, scenario, backend) = classify(&bench_name, &group_name);

            crates
                .entry(crate_name)
                .or_default()
                .entry(scenario)
                .or_default()
                .insert(backend, point_estimate);
        }
    }
    Ok(())
}

fn parse_bench_txt(
    path: &Path,
    crates: &mut BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>>,
) -> Result<()> {
    let content = fs::read_to_string(path).context("bench.txt missing")?;
    for line in content.lines() {
        if !line.contains("time:") {
            continue;
        }
        let (crate_name, scenario, backend) = ("unknown".into(), "bench".into(), "rust".into());
        crates
            .entry(crate_name)
            .or_default()
            .entry(scenario)
            .or_default()
            .insert(backend, 0.0);
    }
    Ok(())
}

fn classify(bench_name: &str, group_name: &str) -> (String, String, String) {
    if bench_name.starts_with("ts_") {
        let rest = bench_name.strip_prefix("ts_").unwrap_or(bench_name);
        let mut parts = rest.splitn(2, '_');
        let lang = parts.next().unwrap_or("unknown");
        let scenario = parts.next().unwrap_or("unknown");
        let scenario_clean = scenario
            .split('_')
            .take_while(|s| !s.chars().all(char::is_numeric))
            .collect::<Vec<_>>()
            .join("_");
        let crate_name = "icb-parser".into();
        let backend = format!("ts_{}", lang);
        let scenario_name = match scenario_clean.as_str() {
            "deeply_nested" => "Deeply Nested".into(),
            "many_calls" => "Many Calls".into(),
            "single_large_file" => "Single Large File".into(),
            "real_project" => "Real Project".into(),
            _ => scenario_clean,
        };
        return (crate_name, scenario_name, backend);
    }

    if bench_name.starts_with("rustc_") {
        let rest = bench_name.strip_prefix("rustc_").unwrap();
        let scenario = rest
            .split('_')
            .take_while(|s| !s.chars().all(char::is_numeric))
            .collect::<Vec<_>>()
            .join("_");
        let scenario_name = match scenario.as_str() {
            "deeply_nested" => "Deeply Nested".into(),
            "many_calls" => "Many Calls".into(),
            "single_large_file" => "Single Large File".into(),
            _ => scenario,
        };
        return ("icb-rustc".into(), scenario_name, "rustc".into());
    }

    match group_name {
        "build_graph" => {
            return ("icb-graph".into(), "Build Graph".into(), "rust".into());
        }
        "full_analysis" => {
            return ("icb-graph".into(), "Full Analysis".into(), "rust".into());
        }
        "resolve_calls" => {
            return ("icb-graph".into(), "Resolve Calls".into(), "rust".into());
        }
        "analytics_metrics" => {
            return (
                "icb-server".into(),
                "Analytics Metrics".into(),
                "rust".into(),
            );
        }
        "graph_serialization" => {
            return (
                "icb-server".into(),
                "Graph Serialization".into(),
                "rust".into(),
            );
        }
        "graph_subgraph" => {
            return ("icb-server".into(), "Graph Subgraph".into(), "rust".into());
        }
        _ => {}
    }

    ("unknown".into(), group_name.into(), "unknown".into())
}
