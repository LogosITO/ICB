//! Graph construction and caching logic for the server.

use icb_common::Language;
use icb_graph::cache;
use icb_graph::graph::CodePropertyGraph;
use std::path::{Path, PathBuf};

use crate::display_name;

/// Builds a new graph or loads it from the specified cache file.
///
/// The resulting graph has **readable names** on every node: any Clang USR
/// identifiers are automatically converted via
/// [`display_name::readable_name`].
///
/// When loading from an existing cache, names are cleaned before returning
/// the graph; if any names are changed, the cache is updated so that
/// subsequent loads are instant.
///
/// # Arguments
///
/// * `project` - Path to the project root or a single file.
/// * `language` - Programming language of the source files.
/// * `compile_commands` - Optional path to a `compile_commands.json`.
/// * `cpp_std` - C++ standard version to pass to the parser.
/// * `cache_path` - Path to a cache file for faster reloading.
/// * `no_system_headers` - Whether to exclude system header nodes.
/// * `max_depth` - Maximum directory depth when scanning for files.
///
/// # Errors
///
/// Returns an error if the graph cannot be built or loaded.
pub fn build_or_load_graph(
    project: &Path,
    language: &str,
    compile_commands: Option<&PathBuf>,
    cpp_std: &str,
    cache_path: Option<&PathBuf>,
    no_system_headers: bool,
    max_depth: Option<usize>,
) -> anyhow::Result<CodePropertyGraph> {
    let lang = parse_language(language)?;

    // Try loading from cache first
    if let Some(cache_file) = cache_path {
        if cache_file.exists() {
            log::info!("Loading graph from cache {:?}", cache_file);
            if let Ok(mut g) = cache::load_graph(cache_file) {
                // Normalise names even for cached graphs
                cleanup_node_names(&mut g);
                // Persist the cleaned version so it's ready next time
                if let Err(e) = cache::save_graph(&g, cache_file) {
                    log::warn!("Failed to update cache with clean names: {}", e);
                } else {
                    log::info!("Cache updated with clean names");
                }
                return Ok(g);
            }
        }
    }

    let manager = icb_parser::manager::ParserManager::new();
    let allow_system = !no_system_headers;
    let file_facts = if lang == Language::Cpp {
        if let Some(cdb) = compile_commands {
            let cdb = cdb.canonicalize()?;
            let base_dir = cdb.parent().unwrap_or(Path::new("."));
            icb_clang::project::parse_project(&cdb, base_dir, true, allow_system)?
        } else if project.is_file() {
            let source = std::fs::read_to_string(project)?;
            let args = vec![format!("-std={}", cpp_std)];
            let facts = icb_clang::parser::parse_cpp_file(
                &source,
                &args,
                Some(project.to_str().unwrap_or("unknown")),
                allow_system,
            )?;
            vec![(
                project
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                facts,
            )]
        } else {
            let args = vec![format!("-std={}", cpp_std)];
            icb_clang::project::parse_directory(project, &args, true, max_depth, allow_system)?
        }
    } else if project.is_dir() {
        manager.parse_directory(lang, project)?
    } else {
        let source = std::fs::read_to_string(project)?;
        let facts = if lang == Language::Cpp {
            let args = vec![format!("-std={}", cpp_std)];
            icb_clang::parser::parse_cpp_file(
                &source,
                &args,
                Some(project.to_str().unwrap_or("unknown")),
                allow_system,
            )?
        } else {
            manager.parse_file(lang, &source)?
        };
        vec![(
            project
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            facts,
        )]
    };

    let mut builder = icb_graph::builder::GraphBuilder::new();
    for (_, facts) in file_facts {
        let mut local = icb_graph::builder::GraphBuilder::new();
        local.ingest_file_facts(&facts);
        builder.merge(local);
    }
    builder.resolve_calls();

    let mut cpg = builder.cpg;

    // Convert USR‑based names to readable display names before the graph is
    // consumed by analytics or the API.
    cleanup_node_names(&mut cpg);

    if let Some(cache_file) = cache_path {
        if let Err(e) = cache::save_graph(&cpg, cache_file) {
            log::warn!("Failed to save cache: {}", e);
        }
    }
    Ok(cpg)
}

/// Walks all graph nodes and replaces USR‑encoded names with their
/// human‑readable equivalents.
fn cleanup_node_names(cpg: &mut CodePropertyGraph) {
    for node in cpg.graph.node_weights_mut() {
        // Очищаем name
        if let Some(ref name) = node.name {
            let cleaned = display_name::readable_name(name);
            if cleaned != *name {
                node.name = Some(cleaned);
            }
        }

        if node.kind == icb_common::NodeKind::Function || node.kind == icb_common::NodeKind::Class {
            if let Some(ref usr) = node.usr {
                if usr.starts_with("c:") {
                    let cleaned = display_name::readable_name(usr);
                    if cleaned != *usr {
                        node.usr = Some(cleaned);
                    }
                }
            }
        }
    }
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
