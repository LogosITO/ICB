//! Collects multiple `bencher`-formatted benchmark outputs and produces a
//! rich JSON object with per‑scenario grouping, metadata, and compatibility
//! with the legacy flat format.
//!
//! Usage:
//! ```bash
//! cargo run -p generate-bench-json -- bench_clang.txt bench_graph.txt … > latest.json
//! ```

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

    let mut scenarios: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    let mut legacy: BTreeMap<String, Vec<LegacyEntry>> = BTreeMap::new();

    for path in &args[1..] {
        let file = fs::File::open(path)?;
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            if let Some((name, ns)) = parse_bencher_line(&line) {
                let (scenario, backend) = classify(&name);
                scenarios.entry(scenario).or_default().insert(backend, ns);
                legacy.entry(name.clone()).or_default().push(LegacyEntry {
                    name: name.clone(),
                    time_ns: ns,
                });
            }
        }
    }

    let metadata = Metadata {
        date: chrono::Utc::now().to_rfc3339(),
        commit: std::env::var("GITHUB_SHA").unwrap_or_default(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let output = Output {
        metadata,
        scenarios,
        legacy: legacy
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().next().unwrap()))
            .collect(),
    };

    serde_json::to_writer(io::stdout(), &output)?;
    Ok(())
}

#[derive(Serialize)]
struct Output {
    metadata: Metadata,
    scenarios: BTreeMap<String, BTreeMap<String, f64>>,
    legacy: BTreeMap<String, LegacyEntry>,
}

#[derive(Serialize)]
struct Metadata {
    date: String,
    commit: String,
    version: String,
}

#[derive(Serialize, Clone)]
struct LegacyEntry {
    name: String,
    time_ns: f64,
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
    let ns_str = rest.split_whitespace().next()?;
    let ns: f64 = ns_str.parse().ok()?;
    Some((name, ns))
}

/// Map a benchmark name to a (scenario, backend) pair.
fn classify(name: &str) -> (String, String) {
    // Examples:
    // single_large_file_1000_funcs (Clang) -> ("Single Large File", "Clang")
    // ts_cpp_large_file_1000_funcs -> ("Single Large File", "tree-sitter C++")
    // ts_go_large_file_1000_funcs -> ("Single Large File", "tree-sitter Go")
    // ts_ruby_large_file_1000_funcs -> ("Single Large File", "tree-sitter Ruby")
    // deeply_nested_5_levels (Clang) -> ("Deeply Nested", "Clang")
    // ts_cpp_deeply_nested_5_levels -> ("Deeply Nested", "tree-sitter C++")
    // many_calls_10000 (Clang) -> ("Many Calls", "Clang")
    // ts_cpp_many_calls_10000 -> ("Many Calls", "tree-sitter C++")
    // with_system_headers / without_system_headers -> ("System Headers", ...)
    // build_graph_*, resolve_calls_*, full_analysis_*, analytics_metrics_*, graph_serialization_*, graph_subgraph_*, ...
    // We'll map to a human-readable scenario name and a backend label.

    if name.starts_with("single_large_file") {
        return ("Single Large File".into(), "Clang".into());
    }
    if name.starts_with("ts_cpp_single_large_file") || name.starts_with("ts_cpp_large_file") {
        return ("Single Large File".into(), "tree-sitter C++".into());
    }
    if name.starts_with("ts_go_large_file") || name.starts_with("ts_go_single_large_file") {
        return ("Single Large File".into(), "tree-sitter Go".into());
    }
    if name.starts_with("ts_ruby_large_file") || name.starts_with("ts_ruby_single_large_file") {
        return ("Single Large File".into(), "tree-sitter Ruby".into());
    }

    if name.starts_with("deeply_nested") {
        return ("Deeply Nested".into(), "Clang".into());
    }
    if name.starts_with("ts_cpp_deeply_nested") {
        return ("Deeply Nested".into(), "tree-sitter C++".into());
    }
    if name.starts_with("ts_go_deeply_nested") {
        return ("Deeply Nested".into(), "tree-sitter Go".into());
    }
    if name.starts_with("ts_ruby_deeply_nested") {
        return ("Deeply Nested".into(), "tree-sitter Ruby".into());
    }

    if name.starts_with("many_calls") {
        return ("Many Calls".into(), "Clang".into());
    }
    if name.starts_with("ts_cpp_many_calls") {
        return ("Many Calls".into(), "tree-sitter C++".into());
    }
    if name.starts_with("ts_go_many_calls") {
        return ("Many Calls".into(), "tree-sitter Go".into());
    }
    if name.starts_with("ts_ruby_many_calls") {
        return ("Many Calls".into(), "tree-sitter Ruby".into());
    }

    if name.starts_with("with_system_headers") {
        return ("System Headers".into(), "Clang (with)".into());
    }
    if name.starts_with("without_system_headers") {
        return ("System Headers".into(), "Clang (without)".into());
    }

    if name.starts_with("build_graph") {
        return ("Graph Build".into(), "icb-graph".into());
    }
    if name.starts_with("resolve_calls") {
        return ("Resolve Calls".into(), "icb-graph".into());
    }
    if name.starts_with("full_analysis") {
        return ("Full Analysis".into(), "icb-graph".into());
    }
    if name.starts_with("analytics_metrics") {
        return ("Analytics Metrics".into(), "icb-server".into());
    }
    if name.starts_with("graph_serialization") {
        return ("Graph Serialization".into(), "icb-server".into());
    }
    if name.starts_with("subgraph_by_kind") {
        return ("Subgraph Extraction".into(), "icb-server".into());
    }

    // Fallback: use the first part of the name as scenario, backend "unknown"
    let scenario = name.split('_').next().unwrap_or("other").to_string();
    (scenario, "unknown".into())
}
