//! Graph construction and caching logic for the server.
//!
//! # Overview
//!
//! This module is responsible for turning raw parser facts into a fully
//! resolved [`CodePropertyGraph`].  It supports three distinct workflows:
//!
//! * **build** – parse a project or a single file and construct the graph
//!   from scratch.
//! * **cache** – serialise the graph to a compressed binary format using
//!   [`icb_graph::cache`] so that subsequent runs can skip parsing
//!   entirely.
//! * **load** – restore a previously cached graph, automatically cleaning
//!   up node names.
//!
//! # Robust directory traversal
//!
//! When `project` is a directory, the module walks it recursively,
//! attempts to read each file as UTF‑8, and **silently skips** files that
//! cannot be decoded (binary files).  Only valid source files are passed
//! to the parser manager.  This makes it safe to point the server at any
//! real‑world project without pre‑filtering.
//!
//! # Auto‑detection of language
//!
//! When `language` is `"auto"`, the module scans the project directory
//! and picks the dominant language based on file extensions.  Unknown
//! extensions fall back to the universal heuristic parser.

use icb_common::Language;
use icb_graph::cache;
use icb_graph::graph::CodePropertyGraph;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::display_name;

/// Builds a new graph or loads it from the specified cache file.
///
/// If `language` is `"auto"`, the dominant language is guessed from the
/// file extensions inside `project`.
///
/// # Arguments
///
/// * `project` - Path to the project root or a single file.
/// * `language` - Programming language or `"auto"`.
/// * `cache_path` - Optional path to a cache file.
pub fn build_or_load_graph(
    project: &Path,
    language: &str,
    cache_path: Option<&PathBuf>,
) -> anyhow::Result<CodePropertyGraph> {
    let lang = if language == "auto" {
        detect_language_from_project(project)
    } else {
        parse_language(language)?
    };

    // Try loading from cache first
    if let Some(cache_file) = cache_path {
        if cache_file.exists() {
            log::info!("Loading graph from cache {:?}", cache_file);
            if let Ok(mut g) = cache::load_graph(cache_file) {
                display_name::cleanup_node_names(&mut g);
                if let Err(e) = cache::save_graph(&g, cache_file) {
                    log::warn!("Failed to update cache with clean names: {}", e);
                }
                return Ok(g);
            }
        }
    }

    let manager = icb_parser::manager::ParserManager::new();

    let file_facts = if project.is_dir() {
        let mut results = Vec::new();
        for entry in WalkDir::new(project)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            // Skip files that are not readable as UTF‑8 (binary files)
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => {
                    log::debug!("Skipping non‑UTF‑8 file: {}", path.display());
                    continue;
                }
            };
            // Attempt to parse the file; skip silently on parse errors
            let facts = match manager.parse_file(lang, &source) {
                Ok(facts) => facts,
                Err(e) => {
                    log::debug!("Skipping unparseable file {}: {}", path.display(), e);
                    continue;
                }
            };
            let rel = path
                .strip_prefix(project)
                .unwrap_or(path)
                .display()
                .to_string();
            results.push((rel, facts));
        }
        results
    } else {
        let source = match std::fs::read_to_string(project) {
            Ok(s) => s,
            Err(e) => {
                anyhow::bail!("Cannot read file {}: {}", project.display(), e);
            }
        };
        let facts = manager.parse_file(lang, &source)?;
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
        let filtered: Vec<_> = facts
            .into_iter()
            .filter(|f| {
                matches!(
                    f.kind,
                    icb_common::NodeKind::Function
                        | icb_common::NodeKind::Class
                        | icb_common::NodeKind::CallSite
                )
            })
            .collect();
        let mut local = icb_graph::builder::GraphBuilder::new();
        local.ingest_file_facts(&filtered);
        builder.merge(local);
    }
    builder.resolve_calls();

    let mut cpg = builder.cpg;
    display_name::cleanup_node_names(&mut cpg);

    if let Some(cache_file) = cache_path {
        if let Err(e) = cache::save_graph(&cpg, cache_file) {
            log::warn!("Failed to save cache: {}", e);
        }
    }
    Ok(cpg)
}

fn parse_language(s: &str) -> anyhow::Result<Language> {
    match s.to_lowercase().as_str() {
        "python" => Ok(Language::Python),
        "cpp" | "c++" => Ok(Language::CppTreeSitter),
        "rust" => Ok(Language::Rust),
        "javascript" | "js" => Ok(Language::JavaScript),
        "go" => Ok(Language::Go),
        "ruby" => Ok(Language::Ruby),
        _ => Ok(Language::Unknown),
    }
}

/// Recursively scan a directory and guess the dominant language.
///
/// Counts file extensions up to depth 3 and returns the corresponding
/// [`Language`].  If no files are found or the dominant extension is
/// unknown, [`Language::Unknown`] is returned, which triggers the
/// universal heuristic parser.
pub fn detect_language_from_project(path: &Path) -> Language {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            *counts.entry(ext.to_lowercase()).or_insert(0) += 1;
        }
    }
    let dominant = counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(ext, _)| ext);
    match dominant.as_deref() {
        Some("py") => Language::Python,
        Some("c") | Some("cpp") | Some("cc") | Some("cxx") | Some("h") | Some("hpp") => {
            Language::CppTreeSitter
        }
        Some("go") => Language::Go,
        Some("rb") => Language::Ruby,
        Some("rs") => Language::Rust,
        Some("js") | Some("ts") | Some("jsx") | Some("tsx") => Language::JavaScript,
        _ => Language::Unknown,
    }
}
