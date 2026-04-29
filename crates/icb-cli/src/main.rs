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
    /// Source file to analyze (cannot be used with --project)
    path: Option<PathBuf>,

    /// Programming language (required if --path is given)
    #[arg(short, long)]
    language: Option<String>,

    /// Project directory to analyze (instead of a single file)
    #[arg(short = 'P', long, conflicts_with = "path")]
    project: Option<PathBuf>,

    /// Language for project files (default: python)
    #[arg(short = 'L', long, requires = "project")]
    project_language: Option<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Cli::parse();

    let manager = ParserManager::new();

    if let Some(project_dir) = args.project {
        // ---- project mode ----
        let lang_str = args.project_language.as_deref().unwrap_or("python");
        let lang = match lang_str {
            "python" => Language::Python,
            "rust" => Language::Rust,
            "javascript" => Language::JavaScript,
            _ => anyhow::bail!("Unsupported project language: {}", lang_str),
        };

        let file_facts = manager.parse_directory(lang, &project_dir)?;
        println!("Parsed {} files", file_facts.len());

        let mut global_builder = GraphBuilder::new();
        for (_rel_path, facts) in file_facts {
            // each file could be merged in parallel, but here we merge sequentially for simplicity
            let mut local = GraphBuilder::new();
            local.ingest_file_facts(&facts);
            global_builder.merge(local);
        }

        let cpg = global_builder.cpg;
        println!(
            "Global graph built: {} nodes, {} edges",
            cpg.node_count(),
            cpg.edge_count()
        );

        // Quick overview: count functions and classes
        let functions = query::find_by_kind(&cpg, icb_common::NodeKind::Function);
        let classes = query::find_by_kind(&cpg, icb_common::NodeKind::Class);
        println!("Functions: {}, Classes: {}", functions.len(), classes.len());
        for f in functions.iter().take(5) {
            println!(
                "  - '{}' at line {}",
                f.name.as_deref().unwrap_or("?"),
                f.start_line
            );
        }
        if functions.len() > 5 {
            println!("  ... and {} more", functions.len() - 5);
        }
    } else if let Some(file_path) = args.path {
        // ---- single file mode ----
        let lang_str = args
            .language
            .ok_or_else(|| anyhow::anyhow!("--language is required for single file"))?;
        let lang = match lang_str.as_str() {
            "python" => Language::Python,
            "rust" => Language::Rust,
            "javascript" => Language::JavaScript,
            _ => anyhow::bail!("Unsupported language: {}", lang_str),
        };

        let source = fs::read_to_string(&file_path)?;
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
    } else {
        anyhow::bail!("Either --path or --project must be provided");
    }

    Ok(())
}
