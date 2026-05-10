use anyhow::Result;
use serde_json::Value;
use std::fs;

const THRESHOLD: f64 = 0.10; // 10%

fn main() -> Result<()> {
    let latest: Value = serde_json::from_str(&fs::read_to_string("docs/bench-data/latest.json")?)?;
    let baseline: Value =
        serde_json::from_str(&fs::read_to_string("docs/bench-data/baseline.json")?)?;

    let l = latest["crates"].as_object().unwrap();
    let b = baseline["crates"].as_object().unwrap();

    let mut failed = false;

    for (krate, ldata) in l {
        if let Some(bdata) = b.get(krate) {
            for (scenario, lsc) in ldata.as_object().unwrap() {
                if let Some(bsc) = bdata.get(scenario) {
                    for (backend, lv) in lsc.as_object().unwrap() {
                        if let Some(bv) = bsc.get(backend) {
                            let lv = lv.as_f64().unwrap();
                            let bv = bv.as_f64().unwrap();

                            let delta = (lv - bv) / bv;

                            if delta > THRESHOLD {
                                println!(
                                    "REGRESSION: {krate} / {scenario} / {backend} : {bv} → {lv} (+{:.2}%)",
                                    delta * 100.0
                                );
                                failed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if failed {
        std::process::exit(1);
    }

    println!("OK: no regressions");
    Ok(())
}
