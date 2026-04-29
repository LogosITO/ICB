use clap::Parser;
use std::path::PathBuf;

use icb_common::Language;
use icb_graph::builder::GraphBuilder;
use icb_graph::{query, visualizer};
use icb_parser::manager::ParserManager;

/// Infinite Code Blueprint – universal code graph CLI.
#[derive(Parser)]
#[command(name = "icb")]
#[command(about = "Infinite Code Blueprint CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Build the Code Property Graph and show basic statistics.
    Analyze {
        /// Source file or directory
        path: PathBuf,
        /// Programming language
        #[arg(short, long)]
        language: String,
    },
    /// Run queries on an already built graph (requires a project directory).
    Query {
        /// Project directory
        project: PathBuf,
        /// Language (default: python)
        #[arg(short, long, default_value = "python")]
        language: String,
        /// Show all functions
        #[arg(long)]
        functions: bool,
        /// Show callers of a function
        #[arg(long)]
        callers: Option<String>,
        /// Show callees of a function
        #[arg(long)]
        callees: Option<String>,
        /// Show unused functions
        #[arg(long)]
        unused: bool,
        /// Export call graph as DOT and print to stdout
        #[arg(long)]
        dot: bool,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let manager = ParserManager::new();

    match cli.command {
        Command::Analyze { path, language } => {
            let lang = parse_language(&language)?;
            let (cpg, _files) = build_project_graph(&manager, lang, &path, true)?;
            println!(
                "Graph: {} nodes, {} edges",
                cpg.node_count(),
                cpg.edge_count()
            );
        }
        Command::Query {
            project,
            language,
            functions,
            callers,
            callees,
            unused,
            dot,
        } => {
            let lang = parse_language(&language)?;
            let (cpg, _files) = build_project_graph(&manager, lang, &project, false)?;

            if functions {
                let funcs = query::find_by_kind(&cpg, icb_common::NodeKind::Function);
                println!("Functions ({})", funcs.len());
                for f in &funcs {
                    println!(
                        "  {} (line {})",
                        f.name.as_deref().unwrap_or("?"),
                        f.start_line
                    );
                }
            }
            if let Some(target) = callers {
                let callers = query::callers_of(&cpg, &target);
                println!("Callers of '{}' ({})", target, callers.len());
                for (caller, _) in &callers {
                    println!(
                        "  {} (line {})",
                        caller.name.as_deref().unwrap_or("?"),
                        caller.start_line
                    );
                }
            }
            if let Some(target) = callees {
                let callees = query::callees_of(&cpg, &target);
                println!("Callees of '{}' ({})", target, callees.len());
                for (callee, _) in &callees {
                    println!(
                        "  {} (line {})",
                        callee.name.as_deref().unwrap_or("?"),
                        callee.start_line
                    );
                }
            }
            if unused {
                let unused = query::unused_functions(&cpg);
                println!("Unused functions ({})", unused.len());
                for f in &unused {
                    println!(
                        "  {} (line {})",
                        f.name.as_deref().unwrap_or("?"),
                        f.start_line
                    );
                }
            }
            if dot {
                println!("{}", visualizer::export_call_dot(&cpg));
            }
        }
    }

    Ok(())
}

fn parse_language(s: &str) -> anyhow::Result<Language> {
    match s {
        "python" => Ok(Language::Python),
        "rust" => Ok(Language::Rust),
        "javascript" => Ok(Language::JavaScript),
        _ => anyhow::bail!("Unsupported language: {}", s),
    }
}

/// Build a global CPG from a project directory (or single file).
/// Returns the graph and how many files were processed.
fn build_project_graph(
    manager: &ParserManager,
    lang: Language,
    path: &std::path::Path,
    show_progress: bool,
) -> anyhow::Result<(icb_graph::graph::CodePropertyGraph, usize)> {
    let file_facts = if path.is_dir() {
        manager.parse_directory(lang, path)?
    } else {
        let source = std::fs::read_to_string(path)?;
        let facts = manager.parse_file(lang, &source)?;
        vec![(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            facts,
        )]
    };

    let files_count = file_facts.len();
    if show_progress {
        println!("Parsed {} files", files_count);
    }

    let mut global_builder = GraphBuilder::new();
    for (_rel_path, facts) in file_facts {
        let mut local = GraphBuilder::new();
        local.ingest_file_facts(&facts);
        global_builder.merge(local);
    }
    global_builder.resolve_calls();

    Ok((global_builder.cpg, files_count))
}
