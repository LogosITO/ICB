//! Collects multiple `bencher`-formatted benchmark outputs and produces a
//! single JSON object keyed by benchmark group.
//!
//! Each input file should contain lines like:
//! ```text
//! test some_bench ... bench:  1234567 ns/iter (+/- 12345)
//! ```
//!
//! Usage:
//! ```bash
//! cargo run -p generate-bench-json -- bench_clang.txt bench_graph.txt … > latest.json
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};

fn main() -> anyhow::Result<()> {
    let mut groups: BTreeMap<String, Vec<BenchEntry>> = BTreeMap::new();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: {} <file1> [file2 ...]", args[0]);
    }

    for path in &args[1..] {
        let file = fs::File::open(path)?;
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            if let Some(entry) = parse_bencher_line(&line) {
                groups.entry(entry.group.clone()).or_default().push(entry);
            }
        }
    }

    serde_json::to_writer(io::stdout(), &groups)?;
    Ok(())
}

#[derive(serde::Serialize)]
struct BenchEntry {
    name: String,
    group: String,
    time_ns: f64,
}

fn parse_bencher_line(line: &str) -> Option<BenchEntry> {
    let line = line.trim();
    if !line.starts_with("test ") {
        return None;
    }
    let rest = line.strip_prefix("test ")?;
    let (name, rest) = rest.split_once(" ... bench:")?;
    let name = name.trim().to_string();
    let rest = rest.trim();
    // Extract the number before " ns/iter"
    let ns_str = rest.split_whitespace().next()?;
    let ns: f64 = ns_str.parse().ok()?;
    // Derive a group name from the benchmark name (e.g., first segment)
    let group = name.split('_').next().unwrap_or("other").to_string();
    Some(BenchEntry {
        name,
        group,
        time_ns: ns,
    })
}
