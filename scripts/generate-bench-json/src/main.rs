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

        for line in io::BufReader::new(file).lines() {
            let line = line?;

            if let Some((name, ns)) = parse_bencher_line(&line) {
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

fn parse_bencher_line(line: &str) -> Option<(String, f64)> {
    let line = line.trim();

    if !line.starts_with("test ") {
        return None;
    }

    let rest = line.strip_prefix("test ")?;
    let (name, rest) = rest.split_once(" ... bench:")?;

    let name = name.trim().to_string();
    let rest = rest.trim();

    // Example:
    // "12,345 ns/iter (+/- 123)"
    let raw = rest.split_whitespace().next()?;

    let cleaned = raw.replace([',', '_'], "");

    let ns: f64 = cleaned.parse().ok()?;

    Some((name, ns))
}

fn classify(name: &str) -> (String, String, String) {
    // icb-clang
    if name.starts_with("single_large_file") {
        return (
            "icb-clang".into(),
            "Single Large File".into(),
            "Clang".into(),
        );
    }

    if name.starts_with("deeply_nested") {
        return ("icb-clang".into(), "Deeply Nested".into(), "Clang".into());
    }

    if name.starts_with("many_calls") {
        return ("icb-clang".into(), "Many Calls".into(), "Clang".into());
    }

    if name.starts_with("system_headers_") {
        let backend = if name.ends_with("_on") {
            "with system"
        } else {
            "without system"
        };

        return ("icb-clang".into(), "System Headers".into(), backend.into());
    }

    // icb-graph
    if name.starts_with("build_graph") {
        return ("icb-graph".into(), "Graph Build".into(), "graph".into());
    }

    if name.starts_with("resolve_calls") {
        return ("icb-graph".into(), "Resolve Calls".into(), "graph".into());
    }

    if name.starts_with("full_analysis") {
        return ("icb-graph".into(), "Full Analysis".into(), "graph".into());
    }

    // icb-server
    if name.starts_with("analytics_metric")
        || name.starts_with("function_metrics")
        || name.starts_with("class_metrics")
        || name.starts_with("file_metrics")
    {
        return ("icb-server".into(), "Metrics".into(), "server".into());
    }

    if name.starts_with("json_serialize") || name.starts_with("graph_serialization") {
        return (
            "icb-server".into(),
            "Graph Serialization".into(),
            "server".into(),
        );
    }

    if name.starts_with("subgraph_") || name.starts_with("focal_graph") {
        return (
            "icb-server".into(),
            "Subgraph Extraction".into(),
            "server".into(),
        );
    }

    if name.starts_with("server_pipeline") || name.starts_with("real_project_pipeline") {
        return (
            "icb-server".into(),
            "Server Pipeline".into(),
            "server".into(),
        );
    }

    // icb-parser (tree-sitter)
    if name.starts_with("ts_") {
        if let Some(rest) = name.strip_prefix("ts_") {
            let parts: Vec<&str> = rest.splitn(2, '_').collect();

            if parts.len() >= 2 {
                let lang = parts[0];
                let scenario_code = parts[1];

                let scenario = match () {
                    _ if scenario_code.starts_with("large_file")
                        || scenario_code.starts_with("single_large_file") =>
                    {
                        "Single Large File"
                    }

                    _ if scenario_code.starts_with("deeply_nested") => "Deeply Nested",

                    _ if scenario_code.starts_with("many_calls") => "Many Calls",

                    _ if scenario_code.starts_with("real_project") => "Real Project",

                    _ => scenario_code,
                };

                let backend = format!("tree-sitter {}", lang);

                return ("icb-parser".into(), scenario.to_string(), backend);
            }
        }
    }

    // icb-rustc
    if name.starts_with("rustc_") {
        let rest = name.strip_prefix("rustc_").unwrap();

        if let Some(first_underscore) = rest.find('_') {
            let scenario_code = &rest[first_underscore + 1..];

            let scenario = match () {
                _ if scenario_code.starts_with("large_file")
                    || scenario_code.starts_with("single_large_file") =>
                {
                    "Single Large File"
                }

                _ if scenario_code.starts_with("deeply_nested") => "Deeply Nested",

                _ if scenario_code.starts_with("many_calls") => "Many Calls",

                _ if scenario_code.starts_with("real_project") => "Real Project",

                _ => scenario_code,
            };

            return ("icb-rustc".into(), scenario.to_string(), "rustc".into());
        }
    }

    ("unknown".into(), name.to_string(), "unknown".into())
}
