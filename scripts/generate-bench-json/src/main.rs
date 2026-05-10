use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        anyhow::bail!("Usage: {} <bench.txt>", args[0]);
    }

    let file = fs::File::open(&args[1])?;
    let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;

    let mut crates: BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>> = BTreeMap::new();

    let mut current_name: Option<String> = None;

    for line in lines {
        let line = line.trim();

        // capture benchmark name
        if is_benchmark_name(line) {
            current_name = Some(line.to_string());
            continue;
        }

        // parse timing line
        if line.contains("time:") {
            if let Some(name) = &current_name {
                if let Some(ns) = parse_time(line) {
                    let (krate, scenario, backend) = classify(name);

                    crates
                        .entry(krate)
                        .or_default()
                        .entry(scenario)
                        .or_default()
                        .insert(backend, ns);
                }
            }
        }
    }

    let output = Output {
        metadata: Metadata {
            date: chrono::Utc::now().to_rfc3339(),
            commit: std::env::var("GITHUB_SHA").unwrap_or_default(),
        },
        crates,
    };

    serde_json::to_writer_pretty(io::stdout(), &output)?;

    Ok(())
}

fn classify(name: &str) -> (String, String, String) {
    if name.contains("clang") {
        ("icb-clang".into(), "clang".into(), "clang".into())
    } else if name.contains("graph") {
        ("icb-graph".into(), "graph".into(), "graph".into())
    } else if name.contains("ts_") {
        ("icb-parser".into(), "tree-sitter".into(), "ts".into())
    } else if name.contains("rustc") {
        ("icb-rustc".into(), "rustc".into(), "rustc".into())
    } else if name.contains("server") {
        ("icb-server".into(), "server".into(), "server".into())
    } else {
        ("unknown".into(), name.to_string(), "unknown".into())
    }
}

fn is_benchmark_name(line: &str) -> bool {
    line.starts_with("ts_")
        || line.starts_with("single_large_file")
        || line.starts_with("deeply_nested")
        || line.starts_with("many_calls")
        || line.starts_with("rustc_")
        || line.starts_with("build_graph")
        || line.starts_with("resolve_calls")
        || line.starts_with("full_analysis")
}

fn parse_time(line: &str) -> Option<f64> {
    let line = line.replace("[", "").replace("]", "");

    let parts: Vec<&str> = line.split_whitespace().collect();

    let idx = parts.iter().position(|p| *p == "time:")?;

    let value: f64 = parts.get(idx + 1)?.parse().ok()?;
    let unit = parts.get(idx + 2)?;

    let ns = match *unit {
        "ns" => value,
        "us" | "µs" => value * 1_000.0,
        "ms" => value * 1_000_000.0,
        "s" => value * 1_000_000_000.0,
        _ => return None,
    };

    Some(ns)
}

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
