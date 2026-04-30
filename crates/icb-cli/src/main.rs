use clap::Parser;
use std::fs;
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
        #[arg(long)]
        no_system_headers: bool,
    },
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
        #[arg(long)]
        no_system_headers: bool,
    },
    /// Generate static HTML report with graph and analytics.
    Report {
        project: PathBuf,
        #[arg(short, long)]
        language: String,
        #[arg(long)]
        compile_commands: Option<PathBuf>,
        #[arg(long, default_value = "c++17")]
        cpp_std: String,
        #[arg(long)]
        cache: Option<PathBuf>,
        #[arg(long)]
        no_system_headers: bool,
        #[arg(short, long, default_value = "report.html")]
        output: PathBuf,
    },
    /// Compare two project versions and generate diff HTML report.
    Diff {
        old_project: PathBuf,
        new_project: PathBuf,
        #[arg(short, long)]
        language: String,
        #[arg(long)]
        compile_commands: Option<PathBuf>,
        #[arg(long, default_value = "c++17")]
        cpp_std: String,
        #[arg(long)]
        cache: Option<PathBuf>,
        #[arg(long)]
        no_system_headers: bool,
        #[arg(short, long, default_value = "diff.html")]
        output: PathBuf,
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
            no_system_headers,
        } => {
            let lang = parse_language(&language)?;
            let opts = BuildOptions {
                manager: &manager,
                lang,
                path: &path,
                compile_commands: compile_commands.as_deref(),
                cpp_std: &cpp_std,
                cache_path: cache_path.as_deref(),
                show_progress: true,
                no_system_headers,
            };
            let (cpg, _) = build_or_load_graph(opts)?;
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
            no_system_headers,
        } => {
            let lang = parse_language(&language)?;
            let opts = BuildOptions {
                manager: &manager,
                lang,
                path: &project,
                compile_commands: compile_commands.as_deref(),
                cpp_std: &cpp_std,
                cache_path: cache_path.as_deref(),
                show_progress: false,
                no_system_headers,
            };
            let (cpg, _) = build_or_load_graph(opts)?;
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
        Command::Report {
            project,
            language,
            compile_commands,
            cpp_std,
            cache: cache_path,
            no_system_headers,
            output,
        } => {
            let lang = parse_language(&language)?;
            let opts = BuildOptions {
                manager: &manager,
                lang,
                path: &project,
                compile_commands: compile_commands.as_deref(),
                cpp_std: &cpp_std,
                cache_path: cache_path.as_deref(),
                show_progress: true,
                no_system_headers,
            };
            let (cpg, _) = build_or_load_graph(opts)?;
            let html = icb_report::report::generate_report(&cpg, &project.display().to_string())?;
            fs::write(&output, html)?;
            println!("Report written to {:?}", output);
        }
        Command::Diff {
            old_project,
            new_project,
            language,
            compile_commands,
            cpp_std,
            cache: cache_path,
            no_system_headers,
            output,
        } => {
            let lang = parse_language(&language)?;
            let opts_old = BuildOptions {
                manager: &manager,
                lang,
                path: &old_project,
                compile_commands: compile_commands.as_deref(),
                cpp_std: &cpp_std,
                cache_path: cache_path.as_deref(),
                show_progress: true,
                no_system_headers,
            };
            let (old_cpg, _) = build_or_load_graph(opts_old)?;
            let opts_new = BuildOptions {
                manager: &manager,
                lang,
                path: &new_project,
                compile_commands: compile_commands.as_deref(),
                cpp_std: &cpp_std,
                cache_path: cache_path.as_deref(),
                show_progress: true,
                no_system_headers,
            };
            let (new_cpg, _) = build_or_load_graph(opts_new)?;
            let html = icb_report::diff::generate_diff(&old_cpg, &new_cpg, "Project")?;
            fs::write(&output, html)?;
            println!("Diff written to {:?}", output);
        }
    }
    Ok(())
}

struct BuildOptions<'a> {
    manager: &'a ParserManager,
    lang: Language,
    path: &'a Path,
    compile_commands: Option<&'a Path>,
    cpp_std: &'a str,
    cache_path: Option<&'a Path>,
    show_progress: bool,
    no_system_headers: bool,
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
    opts: BuildOptions,
) -> anyhow::Result<(icb_graph::graph::CodePropertyGraph, usize)> {
    if let Some(cache_file) = opts.cache_path {
        if cache_file.exists() {
            if let Ok(cpg) = cache::load_graph(cache_file) {
                return Ok((cpg, 0));
            }
        }
    }
    let file_facts = build_file_facts(
        opts.manager,
        opts.lang,
        opts.path,
        opts.compile_commands,
        opts.cpp_std,
        opts.no_system_headers,
    )?;
    let count = file_facts.len();
    if opts.show_progress {
        println!("Parsed {} files", count);
    }
    let mut builder = GraphBuilder::new();
    for (_, facts) in file_facts {
        let mut local = GraphBuilder::new();
        local.ingest_file_facts(&facts);
        builder.merge(local);
    }
    builder.resolve_calls();
    let cpg = builder.cpg;
    if let Some(cache_file) = opts.cache_path {
        if let Err(e) = cache::save_graph(&cpg, cache_file) {
            log::warn!("Failed to save cache: {}", e);
        }
    }
    Ok((cpg, count))
}

fn build_file_facts(
    manager: &ParserManager,
    lang: Language,
    path: &Path,
    compile_commands: Option<&Path>,
    cpp_std: &str,
    no_system_headers: bool,
) -> anyhow::Result<Vec<(String, Vec<icb_parser::facts::RawNode>)>> {
    let allow_system = !no_system_headers;
    if lang == Language::Cpp {
        if let Some(cdb) = compile_commands {
            let cdb = cdb.canonicalize()?;
            let base_dir = cdb.parent().unwrap_or(Path::new("."));
            Ok(icb_clang::project::parse_project(
                &cdb,
                base_dir,
                true,
                allow_system,
            )?)
        } else if path.is_file() {
            let source = std::fs::read_to_string(path)?;
            let args = vec![format!("-std={}", cpp_std)];
            let facts = icb_clang::parser::parse_cpp_file(
                &source,
                &args,
                Some(path.to_str().unwrap()),
                allow_system,
            )?;
            Ok(vec![(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                facts,
            )])
        } else {
            Ok(icb_clang::project::parse_directory(
                path,
                &[format!("-std={}", cpp_std)],
                true,
                None,
                allow_system,
            )?)
        }
    } else if path.is_dir() {
        Ok(manager.parse_directory(lang, path)?)
    } else {
        let source = std::fs::read_to_string(path)?;
        let facts = manager.parse_file(lang, &source)?;
        Ok(vec![(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            facts,
        )])
    }
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
