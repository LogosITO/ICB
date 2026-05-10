use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
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
    let root = Path::new("target/criterion");

    let mut crates: BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>> = BTreeMap::new();

    // 🔥 SAFETY: CI guard
    if !root.exists() {
        eprintln!("⚠️ target/criterion not found — skipping (bench not executed)");
        let output = Output {
            metadata: Metadata {
                date: chrono::Utc::now().to_rfc3339(),
                commit: std::env::var("GITHUB_SHA").unwrap_or_default(),
            },
            crates,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    for bench_dir in fs::read_dir(root)? {
        let bench_dir = bench_dir?;
        let bench_name = bench_dir.file_name().to_string_lossy().to_string();

        // 🔥 IMPORTANT: nested Criterion structure
        for case_dir in fs::read_dir(bench_dir.path())? {
            let case_dir = case_dir?;

            let path = case_dir.path().join("new").join("estimates.json");

            if !path.exists() {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;

            let ns = json["mean"]["point_estimate"].as_f64().unwrap_or(0.0);

            let (krate, scenario, backend) = classify(&bench_name);

            crates
                .entry(krate)
                .or_default()
                .entry(scenario)
                .or_default()
                .insert(backend, ns);
        }
    }

    let output = Output {
        metadata: Metadata {
            date: chrono::Utc::now().to_rfc3339(),
            commit: std::env::var("GITHUB_SHA").unwrap_or_default(),
        },
        crates,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

fn classify(name: &str) -> (String, String, String) {
    if name.contains("single_large_file") {
        ("icb-parser".into(), "Single Large File".into(), "ts".into())
    } else if name.contains("deeply_nested") {
        ("icb-parser".into(), "Deeply Nested".into(), "ts".into())
    } else if name.contains("many_calls") {
        ("icb-parser".into(), "Many Calls".into(), "ts".into())
    } else if name.contains("real_project") {
        ("icb-parser".into(), "Real Project".into(), "ts".into())
    } else if name.contains("clang") {
        ("icb-clang".into(), "clang".into(), "clang".into())
    } else if name.contains("graph") {
        ("icb-graph".into(), "graph".into(), "graph".into())
    } else if name.contains("rustc") {
        ("icb-rustc".into(), "rustc".into(), "rustc".into())
    } else if name.contains("server") {
        ("icb-server".into(), "server".into(), "server".into())
    } else {
        ("unknown".into(), name.to_string(), "unknown".into())
    }
}
