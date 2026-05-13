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
        &args[1]
    } else {
        "target/criterion"
    };

    let mut crates: BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>> = BTreeMap::new();

    if Path::new(input).join("new").exists() {
        parse_criterion(input, &mut crates)?;
    } else {
        parse_bench_txt(input, &mut crates)?;
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

fn parse_criterion(
    root: &str,
    crates: &mut BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>>,
) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        let path = entry.path().join("new").join("estimates.json");
        if !path.exists() {
            continue;
        }

        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;

        let ns = json["mean"]["point_estimate"].as_f64().unwrap_or(0.0);

        let (krate, scenario, backend) = classify(&name);

        crates
            .entry(krate)
            .or_default()
            .entry(scenario)
            .or_default()
            .insert(backend, ns);
    }

    Ok(())
}

fn parse_bench_txt(
    path: &str,
    crates: &mut BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>>,
) -> Result<()> {
    let content = fs::read_to_string(path).context("bench.txt missing")?;

    for line in content.lines() {
        if !line.contains("time:") {
            continue;
        }

        // fallback dummy parsing
        let (krate, scenario, backend) = ("unknown".into(), "bench".into(), "rust".into());

        crates
            .entry(krate)
            .or_default()
            .entry(scenario)
            .or_default()
            .insert(backend, 0.0);
    }

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
    } else {
        ("unknown".into(), name.into(), "unknown".into())
    }
}
