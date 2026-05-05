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

fn classify(name: &str) -> (String, String) {
    // Clang benches
    if name.starts_with("single_large_file") {
        return ("Single Large File".into(), "Clang".into());
    }
    if name.starts_with("deeply_nested") {
        return ("Deeply Nested".into(), "Clang".into());
    }
    if name.starts_with("many_calls") {
        return ("Many Calls".into(), "Clang".into());
    }

    // Tree‑sitter benches
    if name.starts_with("ts_cpp_") {
        return classify_ts("C++", name);
    }
    if name.starts_with("ts_go_") {
        return classify_ts("Go", name);
    }
    if name.starts_with("ts_ruby_") {
        return classify_ts("Ruby", name);
    }

    // System headers
    if name.starts_with("with_system_headers") {
        return ("System Headers".into(), "Clang (with)".into());
    }
    if name.starts_with("without_system_headers") {
        return ("System Headers".into(), "Clang (without)".into());
    }

    // Server metrics
    if name.starts_with("class_metrics") {
        return ("Class Metrics".into(), "icb-server".into());
    }
    if name.starts_with("file_metrics") {
        return ("File Metrics".into(), "icb-server".into());
    }
    if name.starts_with("focal_graph") {
        return ("Focal Graph".into(), "icb-server".into());
    }
    if name.starts_with("function_metrics") {
        return ("Function Metrics".into(), "icb-server".into());
    }
    if name.starts_with("graph_json_serialize") {
        return ("Graph Serialization".into(), "icb-server".into());
    }
    if name.starts_with("subgraph_by_kind") {
        return ("Subgraph Extraction".into(), "icb-server".into());
    }

    // Graph benches
    if name.starts_with("build_graph") {
        return ("Graph Build".into(), "icb-graph".into());
    }
    if name.starts_with("resolve_calls") {
        return ("Resolve Calls".into(), "icb-graph".into());
    }
    if name.starts_with("full_analysis") {
        return ("Full Analysis".into(), "icb-graph".into());
    }

    let scenario = name.split('_').next().unwrap_or("other").to_string();
    (scenario, "unknown".into())
}

fn classify_ts(lang: &str, name: &str) -> (String, String) {
    let backend = format!("tree-sitter {}", lang);
    let rest = name
        .strip_prefix(&format!("ts_{}_", lang.to_lowercase()))
        .unwrap_or(name);
    if rest.starts_with("large_file") || rest.starts_with("single_large_file") {
        ("Single Large File".into(), backend)
    } else if rest.starts_with("deeply_nested") {
        ("Deeply Nested".into(), backend)
    } else if rest.starts_with("many_calls") {
        ("Many Calls".into(), backend)
    } else {
        (rest.to_string(), backend)
    }
}
