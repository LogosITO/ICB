use clap::Parser;
use std::path::{Path, PathBuf};

use icb_common::Language;
use icb_graph::builder::GraphBuilder;
use icb_graph::{analysis, cache, query, visualizer};
use icb_parser::manager::ParserManager;

#[derive(Parser)]
#[command(name = "icb")]
#[command(about = "Infinite Code Blueprint CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Build Code Property Graph and show basic statistics.
    Analyze {
        path: PathBuf,
        #[arg(short, long)]
        language: String,
        #[arg(long)]
        compile_commands: Option<PathBuf>,
        #[arg(long, default_value = "c++17")]
        cpp_std: String,
        #[arg(long)]
        cache: Option<PathBuf>,
    },
    /// Run queries on a project directory.
    Query {
        project: PathBuf,
        #[arg(short, long, default_value = "python")]
        language: String,
        #[arg(long)]
        compile_commands: Option<PathBuf>,
        #[arg(long, default_value = "c++17")]
        cpp_std: String,
        #[arg(long)]
        functions: bool,
        #[arg(long)]
        callers: Option<String>,
        #[arg(long)]
        callees: Option<String>,
        #[arg(long)]
        unused: bool,
        #[arg(long)]
        dot: bool,
        #[arg(long)]
        cycles: bool,
        #[arg(long)]
        dead_code: bool,
        #[arg(long, default_value = "main", requires = "dead_code")]
        entries: String,
        #[arg(long)]
        complexity: bool,
        #[arg(long, default_value = "20", requires = "complexity")]
        threshold: usize,
        #[arg(long)]
        cache: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let manager = ParserManager::new();

    match cli.command {
        Command::Analyze {
            path,
            language,
            compile_commands,
            cpp_std,
            cache: cache_path,
        } => {
            let lang = parse_language(&language)?;
            let (cpg, _files) = build_or_load_graph(
                &manager,
                lang,
                &path,
                compile_commands.as_deref(),
                &cpp_std,
                cache_path.as_deref(),
                true,
            )?;
            println!(
                "Graph: {} nodes, {} edges",
                cpg.node_count(),
                cpg.edge_count()
            );
        }
        Command::Query {
            project,
            language,
            compile_commands,
            cpp_std,
            functions,
            callers,
            callees,
            unused,
            dot,
            cycles,
            dead_code,
            entries,
            complexity,
            threshold,
            cache: cache_path,
        } => {
            let lang = parse_language(&language)?;
            let (cpg, _files) = build_or_load_graph(
                &manager,
                lang,
                &project,
                compile_commands.as_deref(),
                &cpp_std,
                cache_path.as_deref(),
                false,
            )?;

            if functions {
                print_functions(&cpg);
            }
            if let Some(target) = callers {
                print_callers(&cpg, &target);
            }
            if let Some(target) = callees {
                print_callees(&cpg, &target);
            }
            if unused {
                print_unused(&cpg);
            }
            if dot {
                println!("{}", visualizer::export_call_dot(&cpg));
            }
            if cycles {
                print_cycles(&cpg);
            }
            if dead_code {
                let entry_list: Vec<String> =
                    entries.split(',').map(|s| s.trim().to_string()).collect();
                print_dead_code(&cpg, &entry_list);
            }
            if complexity {
                print_complexity(&cpg, threshold);
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
        "cpp" | "c++" => Ok(Language::Cpp),
        _ => anyhow::bail!("Unsupported language: {}", s),
    }
}

fn build_or_load_graph(
    manager: &ParserManager,
    lang: Language,
    path: &Path,
    compile_commands: Option<&Path>,
    cpp_std: &str,
    cache_path: Option<&Path>,
    show_progress: bool,
) -> anyhow::Result<(icb_graph::graph::CodePropertyGraph, usize)> {
    if let Some(cache_file) = cache_path {
        if cache_file.exists() {
            log::info!("Loading cached graph from {:?}", cache_file);
            match cache::load_graph(cache_file) {
                Ok(cpg) => {
                    log::info!(
                        "Using cached graph ({} nodes, {} edges)",
                        cpg.node_count(),
                        cpg.edge_count()
                    );
                    return Ok((cpg, 0));
                }
                Err(e) => log::warn!("Failed to load cache: {}", e),
            }
        }
    }

    let (cpg, files_count) = build_project_graph(
        manager,
        lang,
        path,
        compile_commands,
        cpp_std,
        show_progress,
    )?;

    if let Some(cache_file) = cache_path {
        if let Err(e) = cache::save_graph(&cpg, cache_file) {
            log::warn!("Failed to save cache: {}", e);
        } else {
            log::info!("Graph cached to {:?}", cache_file);
        }
    }

    Ok((cpg, files_count))
}

fn build_project_graph(
    manager: &ParserManager,
    lang: Language,
    path: &Path,
    compile_commands: Option<&Path>,
    cpp_std: &str,
    show_progress: bool,
) -> anyhow::Result<(icb_graph::graph::CodePropertyGraph, usize)> {
    let file_facts: Vec<(String, Vec<icb_parser::facts::RawNode>)> = if lang == Language::Cpp {
        if let Some(cdb) = compile_commands {
            let cdb = cdb.canonicalize()?;
            let base_dir = cdb.parent().unwrap_or(Path::new("."));
            icb_clang::project::parse_project(&cdb, base_dir, true, true)?
        } else if path.is_file() {
            let source = std::fs::read_to_string(path)?;
            let args = vec![format!("-std={}", cpp_std)];
            let facts = icb_clang::parser::parse_cpp_file(
                &source,
                &args,
                Some(
                    path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("unknown"),
                ),
                true,
            )?;
            vec![(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                facts,
            )]
        } else {
            let args = vec![format!("-std={}", cpp_std)];
            icb_clang::project::parse_directory(path, &args, true, None, true)?
        }
    } else if path.is_dir() {
        manager.parse_directory(lang, path)?
    } else {
        let source = std::fs::read_to_string(path)?;
        let facts = if lang == Language::Cpp {
            let args = vec![format!("-std={}", cpp_std)];
            icb_clang::parser::parse_cpp_file(
                &source,
                &args,
                Some(
                    path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("unknown"),
                ),
                true,
            )?
        } else {
            manager.parse_file(lang, &source)?
        };
        vec![(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            facts,
        )]
    };

    let files_count = file_facts.len();
    if show_progress {
        println!("Parsed {} files", files_count);
    }

    let mut global_builder = GraphBuilder::new();
    for (_, facts) in file_facts {
        let mut local = GraphBuilder::new();
        local.ingest_file_facts(&facts);
        global_builder.merge(local);
    }
    global_builder.resolve_calls();

    Ok((global_builder.cpg, files_count))
}

fn print_functions(cpg: &icb_graph::graph::CodePropertyGraph) {
    let funcs = query::find_by_kind(cpg, icb_common::NodeKind::Function);
    println!("Functions ({})", funcs.len());
    for f in &funcs {
        println!(
            "  {} (line {})",
            f.name.as_deref().unwrap_or("?"),
            f.start_line
        );
    }
}

fn print_callers(cpg: &icb_graph::graph::CodePropertyGraph, target: &str) {
    let callers = query::callers_of(cpg, target);
    println!("Callers of '{}' ({})", target, callers.len());
    for (caller, _) in &callers {
        println!(
            "  {} (line {})",
            caller.name.as_deref().unwrap_or("?"),
            caller.start_line
        );
    }
}

fn print_callees(cpg: &icb_graph::graph::CodePropertyGraph, target: &str) {
    let callees = query::callees_of(cpg, target);
    println!("Callees of '{}' ({})", target, callees.len());
    for (callee, _) in &callees {
        println!(
            "  {} (line {})",
            callee.name.as_deref().unwrap_or("?"),
            callee.start_line
        );
    }
}

fn print_unused(cpg: &icb_graph::graph::CodePropertyGraph) {
    let unused = query::unused_functions(cpg);
    println!("Unused functions ({})", unused.len());
    for f in &unused {
        println!(
            "  {} (line {})",
            f.name.as_deref().unwrap_or("?"),
            f.start_line
        );
    }
}

fn print_cycles(cpg: &icb_graph::graph::CodePropertyGraph) {
    let cycles = analysis::detect_call_cycles(cpg);
    println!("Call cycles ({})", cycles.len());
    for cycle in &cycles {
        println!("  Length {}: {}", cycle.length, cycle.functions.join(", "));
    }
}

fn print_dead_code(cpg: &icb_graph::graph::CodePropertyGraph, entries: &[String]) {
    let dead = analysis::detect_dead_code(cpg, entries);
    println!("Dead code from entries {:?} ({})", entries, dead.len());
    for f in &dead {
        println!(
            "  {} (line {})",
            f.name.as_deref().unwrap_or("?"),
            f.start_line
        );
    }
}

fn print_complexity(cpg: &icb_graph::graph::CodePropertyGraph, threshold: usize) {
    let complex = analysis::detect_complex_functions(cpg, threshold);
    println!(
        "Complex functions (threshold {}): {}",
        threshold,
        complex.len()
    );
    for report in &complex {
        println!(
            "  {} (AST nodes: {}, line {})",
            report.function_name, report.ast_node_count, report.start_line
        );
    }
}
