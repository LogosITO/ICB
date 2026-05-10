use anyhow::Result;
#[expect(unused_imports, reason = "serde used in future JSON export pipeline")]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;

const THRESHOLD: f64 = 0.08; // 8% LLVM-style tolerance

#[expect(dead_code, reason = "used in future perf aggregation pipeline")]
struct PerfPoint {
    name: String,
    value: f64,
}

type PerfDB = BTreeMap<String, BTreeMap<String, BTreeMap<String, f64>>>;

fn main() -> Result<()> {
    let latest: Value = serde_json::from_str(&fs::read_to_string("docs/bench-data/latest.json")?)?;
    let baseline: Value =
        serde_json::from_str(&fs::read_to_string("docs/bench-data/baseline.json")?)?;

    let latest = extract(&latest)?;
    let baseline = extract(&baseline)?;

    let report = diff(&baseline, &latest);

    if report.has_regression {
        println!("❌ PERFORMANCE REGRESSION DETECTED\n");

        for r in &report.items {
            println!(
                "{} | {} | {} → {}% ({:.2}ns → {:.2}ns)",
                r.krate, r.scenario, r.backend, r.delta_pct, r.base, r.latest
            );
        }

        std::process::exit(1);
    }

    println!("✅ NO REGRESSIONS");
    Ok(())
}

#[derive(Debug)]
struct DiffItem {
    krate: String,
    scenario: String,
    backend: String,
    base: f64,
    latest: f64,
    delta_pct: f64,
}

struct DiffReport {
    has_regression: bool,
    items: Vec<DiffItem>,
}

fn extract(v: &Value) -> Result<PerfDB> {
    let mut db = PerfDB::new();

    let crates = v["crates"].as_object().unwrap();

    for (krate, data) in crates {
        let scenarios = data.as_object().unwrap();

        for (scenario, backends) in scenarios {
            let backends = backends.as_object().unwrap();

            for (backend, value) in backends {
                let ns = value.as_f64().unwrap();

                db.entry(krate.clone())
                    .or_default()
                    .entry(scenario.clone())
                    .or_default()
                    .insert(backend.clone(), ns);
            }
        }
    }

    Ok(db)
}

fn diff(base: &PerfDB, latest: &PerfDB) -> DiffReport {
    let mut items = vec![];

    for (krate, lscenarios) in latest {
        if let Some(bscenarios) = base.get(krate) {
            for (scenario, lbackends) in lscenarios {
                if let Some(bbackends) = bscenarios.get(scenario) {
                    for (backend, lv) in lbackends {
                        if let Some(bv) = bbackends.get(backend) {
                            let delta = (lv - bv) / bv;

                            if delta > THRESHOLD {
                                items.push(DiffItem {
                                    krate: krate.clone(),
                                    scenario: scenario.clone(),
                                    backend: backend.clone(),
                                    base: *bv,
                                    latest: *lv,
                                    delta_pct: delta * 100.0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    DiffReport {
        has_regression: !items.is_empty(),
        items,
    }
}
