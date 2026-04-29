use clap::Parser;
use std::fs;
use std::path::PathBuf;

use icb_common::Language;
use icb_graph::builder::GraphBuilder;
use icb_graph::query;
use icb_parser::manager::ParserManager;

/// Infinite Code Blueprint – universal code graph CLI.
///
/// Parses source files, builds a Code Property Graph, and runs queries.
#[derive(Parser)]
#[command(name = "icb")]
#[command(about = "Infinite Code Blueprint CLI")]
struct Cli {
    /// Source file to analyze
    path: PathBuf,
    /// Programming language
    #[arg(short, long)]
    language: String,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let lang = match args.language.as_str() {
        "python" => Language::Python,
        "rust" => Language::Rust,
        "javascript" => Language::JavaScript,
        _ => anyhow::bail!("Unsupported language: {}", args.language),
    };

    let source = fs::read_to_string(&args.path)?;
    let manager = ParserManager::new();
    let facts = manager.parse_file(lang, &source)?;

    let mut builder = GraphBuilder::new();
    builder.ingest_file_facts(&facts);
    let cpg = builder.cpg;

    println!(
        "Graph built: {} nodes, {} edges",
        cpg.node_count(),
        cpg.edge_count()
    );

    let functions = query::find_by_kind(&cpg, icb_common::NodeKind::Function);
    for f in functions {
        println!(
            "Function '{}' at line {}",
            f.name.as_deref().unwrap_or("?"),
            f.start_line
        );
    }

    Ok(())
}
