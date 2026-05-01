//! Ad‑hoc profiler for real‑world C/C++ projects.
//!
//! Walks a directory, parses every source file, and prints the number of
//! raw facts per file, sorted by size descending.
//!
//! ```bash
//! cargo run --example profile_project -- ../Vizora
//! ```

use std::env;
use std::path::Path;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let root = env::args().nth(1).unwrap_or_else(|| "../Vizora".into());
    let root = Path::new(&root);
    if !root.exists() {
        anyhow::bail!("Project path {:?} does not exist", root);
    }

    let args: Vec<String> = vec!["-std=c++17".into()];
    let allow_system = false;

    // Show first 10 files to verify discovery
    let mut files_for_preview = Vec::new();
    icb_clang::project::collect_cpp_files_for_preview(root, &mut files_for_preview, None)?;
    println!("First 10 discovered files:");
    for f in files_for_preview.iter().take(10) {
        println!("  {}", f.display());
    }
    println!("Total discovered: {}\n", files_for_preview.len());

    let start = Instant::now();
    let file_facts = icb_clang::project::parse_directory(root, &args, false, None, allow_system)?;
    let parse_duration = start.elapsed();

    let mut entries: Vec<_> = file_facts
        .iter()
        .map(|(path, facts)| (path.clone(), facts.len()))
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    let total_facts: usize = entries.iter().map(|e| e.1).sum();
    println!("Total files parsed: {}", entries.len());
    println!("Total facts: {}", total_facts);
    println!("Parse time: {:.2?}\n", parse_duration);

    if !entries.is_empty() {
        println!("Top 20 files by fact count:");
        for (i, (path, count)) in entries.iter().take(20).enumerate() {
            println!("{:2}. {:6} facts – {}", i + 1, count, path);
        }
    }

    Ok(())
}
