use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        anyhow::bail!("Usage: {} <file1> [file2 ...]", args[0]);
    }

    let mut crates: BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>> = BTreeMap::new();

    for path in &args[1..] {
        eprintln!("Reading benchmark file: {}", path);

        let file = fs::File::open(path)?;

        let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;

        for i in 0..lines.len() {
            if let Some((name, ns)) = parse_benchmark(&lines, i) {
                eprintln!("Parsed benchmark: {} -> {} ns", name, ns);

                let (crate_name, scenario, backend) = classify(&name);

                crates
                    .entry(crate_name)
                    .or_default()
                    .entry(scenario)
                    .or_default()
                    .insert(backend, ns);
            }
        }
    }

    let metadata = Metadata {
        date: chrono::Utc::now().to_rfc3339(),
        commit: std::env::var("GITHUB_SHA").unwrap_or_default(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let output = Output { metadata, crates };

    serde_json::to_writer_pretty(io::stdout(), &output)?;

    Ok(())
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
    version: String,
}

fn parse_benchmark(lines: &[String], index: usize) -> Option<(String, f64)> {
    let line = lines.get(index)?.trim();

    if line.is_empty() {
        return None;
    }

    if line.starts_with("Benchmarking ") {
        return None;
    }

    let next = lines.get(index + 1)?.trim();

    if !next.starts_with("time:") {
        return None;
    }

    let name = line.to_string();

    let start = next.find('[')?;
    let end = next.find(']')?;

    let values = &next[start + 1..end];

    let parts: Vec<&str> = values.split_whitespace().collect();

    if parts.len() < 2 {
        return None;
    }

    let value: f64 = parts[0].replace([',', '_'], "").parse().ok()?;

    let unit = parts[1];

    let ns = match unit {
        "ns" => value,
        "us" | "µs" => value * 1_000.0,
        "ms" => value * 1_000_000.0,
        "s" => value * 1_000_000_000.0,
        _ => return None,
    };

    Some((name, ns))
}

fn classify(name: &str) -> (String, String, String) {
    if name.starts_with("rustc_") {
        let scenario = if name.contains("single_large_file") {
            "Single Large File"
        } else if name.contains("deeply_nested") {
            "Deeply Nested"
        } else if name.contains("many_calls") {
            "Many Calls"
        } else if name.contains("real_project") {
            "Real Project"
        } else {
            name
        };

        return ("icb-rustc".into(), scenario.to_string(), "rustc".into());
    }

    if name.starts_with("single_large_file") {
        return (
            "icb-clang".into(),
            "Single Large File".into(),
            "clang".into(),
        );
    }

    if name.starts_with("deeply_nested") {
        return ("icb-clang".into(), "Deeply Nested".into(), "clang".into());
    }

    if name.starts_with("many_calls") {
        return ("icb-clang".into(), "Many Calls".into(), "clang".into());
    }

    if name.starts_with("system_headers") {
        let backend = if name.ends_with("_on") {
            "with system"
        } else {
            "without system"
        };

        return ("icb-clang".into(), "System Headers".into(), backend.into());
    }

    if name.starts_with("build_graph") {
        return ("icb-graph".into(), "Graph Build".into(), "graph".into());
    }

    if name.starts_with("resolve_calls") {
        return ("icb-graph".into(), "Resolve Calls".into(), "graph".into());
    }

    if name.starts_with("full_analysis") {
        return ("icb-graph".into(), "Full Analysis".into(), "graph".into());
    }

    if name.starts_with("analytics")
        || name.starts_with("function_metrics")
        || name.starts_with("class_metrics")
        || name.starts_with("file_metrics")
    {
        return ("icb-server".into(), "Metrics".into(), "server".into());
    }

    if name.starts_with("graph_serialization") || name.starts_with("json_serialize") {
        return (
            "icb-server".into(),
            "Graph Serialization".into(),
            "server".into(),
        );
    }

    if name.starts_with("graph_subgraph")
        || name.starts_with("subgraph")
        || name.starts_with("focal_graph")
    {
        return (
            "icb-server".into(),
            "Subgraph Extraction".into(),
            "server".into(),
        );
    }

    if name.starts_with("ts_") {
        if let Some(rest) = name.strip_prefix("ts_") {
            let parts: Vec<&str> = rest.splitn(2, '_').collect();

            if parts.len() >= 2 {
                let lang = parts[0];
                let scenario_code = parts[1];

                let scenario = if scenario_code.contains("single_large_file")
                    || scenario_code.contains("large_file")
                {
                    "Single Large File"
                } else if scenario_code.contains("deeply_nested") {
                    "Deeply Nested"
                } else if scenario_code.contains("many_calls") {
                    "Many Calls"
                } else if scenario_code.contains("real_project") {
                    "Real Project"
                } else {
                    scenario_code
                };

                return (
                    "icb-parser".into(),
                    scenario.to_string(),
                    format!("tree-sitter {}", lang),
                );
            }
        }
    }

    ("unknown".into(), name.to_string(), "unknown".into())
}
