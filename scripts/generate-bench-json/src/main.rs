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
        let file = fs::File::open(path)?;
        let lines: Vec<String> = io::BufReader::new(file).lines().collect::<Result<_, _>>()?;

        let mut i = 0;
        while i + 1 < lines.len() {
            if let Some((name, ns)) = parse_benchmark(&lines, i) {
                let (krate, scenario, backend) = classify(&name);

                crates
                    .entry(krate)
                    .or_default()
                    .entry(scenario)
                    .or_default()
                    .insert(backend, ns);
            }
            i += 1;
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

fn parse_benchmark(lines: &[String], index: usize) -> Option<(String, f64)> {
    let line = lines.get(index)?.trim();
    let next = lines.get(index + 1)?.trim();

    if line.is_empty() || !next.starts_with("time:") {
        return None;
    }

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

    Some((line.to_string(), ns))
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
        ("unknown".into(), name.into(), "unknown".into())
    }
}
