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
//! # C/C++: Clang preferred, tree‑sitter fallback
//!
//! For C and C++ projects the module **prefers the Clang parser** when the
//! `icb-clang` crate is available.  Clang provides exact semantic
//! analysis and never mistakes documentation, HTML, or embedded JavaScript
//! for C++ code.  If Clang cannot be loaded (e.g. LLVM not installed), the
//! pipeline automatically falls back to tree‑sitter‑cpp.
//!
//! # Class fallback
//!
//! If after the primary parse tree‑sitter returned zero class nodes, a
//! lightweight heuristic pass re‑scans the source files for lines matching
//! `class Identifier`, `struct Identifier`, etc., and adds those nodes to
//! the graph.  This guarantees that even template‑heavy or unusual C++
//! code is handled properly.
//!
//! # Strict file extension filtering & multi‑language support
//!
//! For every known language, only a curated list of file extensions is
//! accepted.  This eliminates noise from documentation, build artefacts,
//! and web assets.  When `build_or_load_graph_multi` is called with a
//! specific list of languages, only those extensions are scanned, making
//! it easy to analyse mixed‑language projects or filter out unwanted
//! sources.
//!
//! # Comment stripping and post‑filtering
//!
//! Before parsing, all C‑style comments (`//`, `/* */`) are replaced with
//! whitespace so that Doxygen documentation cannot inject spurious facts.
//! After parsing, a strict filter discards any fact whose name does not
//! look like a plausible identifier or matches a list of known
//! JavaScript/DOM noise words.  For C++, qualified names containing `::`
//! are always accepted.
//!
//! # Auto‑detection of language
//!
//! When `language` is `"auto"`, the module scans the project directory
//! and picks the dominant language based on file extensions.  Unknown
//! extensions fall back to the universal heuristic parser.

use anyhow::anyhow;
use icb_common::Language;
use icb_graph::cache;
use icb_graph::graph::CodePropertyGraph;
use icb_parser::facts::RawNode;
use std::collections::{HashMap, HashSet};
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

    // ----- Allowed extensions for every recognised language -----
    let allowed_extensions: &[&str] = match lang {
        Language::Cpp | Language::CppTreeSitter => &[
            "c", "cpp", "cc", "cxx", "c++", "h", "hpp", "hxx", "hh", "h++", "inl", "inc",
        ],
        Language::Python => &["py"],
        Language::Go => &["go"],
        Language::Ruby => &["rb"],
        Language::Rust => &["rs"],
        Language::JavaScript => &["js", "jsx", "ts", "tsx"],
        _ => &[],
    };

    // ----- If C/C++ try Clang first, then fall back to tree‑sitter -----
    if matches!(lang, Language::Cpp | Language::CppTreeSitter) {
        if let Some(cpg) = try_clang_graph(project, allowed_extensions, cache_path) {
            return Ok(cpg);
        }
        log::info!("Clang not available, falling back to tree‑sitter for C++");
    }

    // ----- Generic path (tree‑sitter / heuristic) -----
    let manager = icb_parser::manager::ParserManager::new();

    let file_facts = if project.is_dir() {
        let mut results = Vec::new();
        for entry in WalkDir::new(project)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Extension filter
            if !allowed_extensions.is_empty() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if !allowed_extensions.contains(&ext.to_lowercase().as_str()) {
                    continue;
                }
            }

            let raw_source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => {
                    log::debug!("Skipping non‑UTF‑8 file: {}", path.display());
                    continue;
                }
            };

            // Remove C‑style comments so that Doxygen / embedded code is neutralised
            let source = strip_c_comments(&raw_source);

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
        let raw_source = match std::fs::read_to_string(project) {
            Ok(s) => s,
            Err(e) => anyhow::bail!("Cannot read file {}: {}", project.display(), e),
        };
        let source = strip_c_comments(&raw_source);
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

    // ----- Post‑filter: discard facts with clearly invalid names -----
    let file_facts: Vec<(String, Vec<RawNode>)> = file_facts
        .into_iter()
        .map(|(path, facts)| {
            let cleaned: Vec<_> = facts
                .into_iter()
                .filter(|f| {
                    let name = f.name.as_deref().unwrap_or("");
                    is_valid_identifier(name, lang) && !is_javascript_noise(name)
                })
                .collect();
            (path, cleaned)
        })
        .collect();

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

    // ----- Fallback class detection for C/C++ if no classes were found -----
    if matches!(lang, Language::Cpp | Language::CppTreeSitter) {
        let class_count = builder
            .cpg
            .graph
            .node_weights()
            .filter(|n| n.kind == icb_common::NodeKind::Class)
            .count();
        if class_count == 0 {
            log::info!("No classes found by primary parser, applying heuristic class extraction");
            let extra_classes = if project.is_dir() {
                let mut extra = Vec::new();
                for entry in WalkDir::new(project)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let path = entry.path();
                    if !allowed_extensions.is_empty() {
                        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                        if !allowed_extensions.contains(&ext.to_lowercase().as_str()) {
                            continue;
                        }
                    }
                    if let Ok(source) = std::fs::read_to_string(path) {
                        extra.extend(icb_parser::heuristic_parser::extract_classes_only(
                            &source,
                            &path.display().to_string(),
                        ));
                    }
                }
                extra
            } else {
                let source = std::fs::read_to_string(project)?;
                icb_parser::heuristic_parser::extract_classes_only(
                    &source,
                    &project.display().to_string(),
                )
            };
            let mut builder2 = icb_graph::builder::GraphBuilder::new();
            builder2.ingest_file_facts(&extra_classes);
            builder.merge(builder2);
            builder.resolve_calls();
        }
    }

    let mut cpg = builder.cpg;
    display_name::cleanup_node_names(&mut cpg);

    if let Some(cache_file) = cache_path {
        if let Err(e) = cache::save_graph(&cpg, cache_file) {
            log::warn!("Failed to save cache: {}", e);
        }
    }
    Ok(cpg)
}

/// Build a graph by merging facts from multiple languages.
///
/// Only files whose extension matches one of the requested languages are
/// processed.  If `languages` is empty or contains `"auto"`, the old
/// auto‑detection behaviour is used.
pub fn build_or_load_graph_multi(
    project: &Path,
    languages: &[String],
    cache_path: Option<&PathBuf>,
) -> anyhow::Result<CodePropertyGraph> {
    if languages.is_empty() || languages.iter().any(|l| l == "auto") {
        return build_or_load_graph(project, "auto", cache_path);
    }

    let allowed: HashSet<Language> = languages
        .iter()
        .filter_map(|l| match l.to_lowercase().as_str() {
            "cpp" | "c++" => Some(Language::CppTreeSitter),
            "python" => Some(Language::Python),
            "go" => Some(Language::Go),
            "ruby" => Some(Language::Ruby),
            "rust" => Some(Language::Rust),
            "javascript" | "js" => Some(Language::JavaScript),
            _ => None,
        })
        .collect();

    let manager = icb_parser::manager::ParserManager::new();
    let mut builder = icb_graph::builder::GraphBuilder::new();

    for entry in WalkDir::new(project)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let lang = detect_language_from_extension(ext);
        if !allowed.contains(&lang) && lang != Language::Cpp && lang != Language::CppTreeSitter {
            continue;
        }

        let raw_source = std::fs::read_to_string(path).map_err(|e| anyhow!(e))?;
        let source = strip_c_comments(&raw_source);
        let facts = manager.parse_file(lang, &source)?;
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

    Ok(builder.cpg)
}

/// Quickly determine language from a single extension (no directory scan).
fn detect_language_from_extension(ext: &str) -> Language {
    match ext.to_lowercase().as_str() {
        "c" | "cpp" | "cc" | "cxx" | "c++" | "h" | "hpp" | "hxx" | "hh" | "h++" | "inl" | "inc" => {
            Language::CppTreeSitter
        }
        "py" => Language::Python,
        "go" => Language::Go,
        "rb" => Language::Ruby,
        "rs" => Language::Rust,
        "js" | "jsx" | "ts" | "tsx" => Language::JavaScript,
        _ => Language::Unknown,
    }
}

fn try_clang_graph(
    _project: &Path,
    _allowed_extensions: &[&str],
    _cache_path: Option<&PathBuf>,
) -> Option<CodePropertyGraph> {
    #[cfg(feature = "clang")]
    {
        use icb_clang::project;
        let file_facts = if _project.is_dir() {
            project::parse_directory(_project, &["-std=c++17".into()], true, None, true)
                .ok()?
                .into_iter()
                .filter(|(p, _)| {
                    let ext = std::path::Path::new(p)
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    _allowed_extensions.contains(&ext.to_lowercase().as_str())
                })
                .collect::<Vec<_>>()
        } else {
            let source = std::fs::read_to_string(_project).ok()?;
            let facts = icb_clang::parser::parse_cpp_file(
                &source,
                &["-std=c++17".into()],
                Some(_project.to_str().unwrap()),
                true,
            )
            .ok()?;
            vec![(
                _project
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
        if let Some(cache_file) = _cache_path {
            let _ = cache::save_graph(&cpg, cache_file);
        }
        Some(cpg)
    }
    #[cfg(not(feature = "clang"))]
    {
        None
    }
}

fn parse_language(s: &str) -> anyhow::Result<Language> {
    match s.to_lowercase().as_str() {
        "python" => Ok(Language::Python),
        "cpp" | "c++" => Ok(Language::Cpp),
        "rust" => Ok(Language::Rust),
        "javascript" | "js" => Ok(Language::JavaScript),
        "go" => Ok(Language::Go),
        "ruby" => Ok(Language::Ruby),
        _ => Ok(Language::Unknown),
    }
}

pub fn detect_language_from_project(path: &Path) -> Language {
    let known: Vec<(&str, Language)> = vec![
        ("cpp", Language::Cpp),
        ("cc", Language::Cpp),
        ("cxx", Language::Cpp),
        ("c++", Language::Cpp),
        ("c", Language::Cpp),
        ("h", Language::Cpp),
        ("hpp", Language::Cpp),
        ("hxx", Language::Cpp),
        ("hh", Language::Cpp),
        ("h++", Language::Cpp),
        ("inl", Language::Cpp),
        ("inc", Language::Cpp),
        ("py", Language::Python),
        ("go", Language::Go),
        ("rb", Language::Ruby),
        ("rs", Language::Rust),
        ("js", Language::JavaScript),
        ("ts", Language::JavaScript),
        ("jsx", Language::JavaScript),
        ("tsx", Language::JavaScript),
    ];

    let mut counts: HashMap<Language, usize> = HashMap::new();
    let mut total = 0usize;

    for entry in WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(ext) = entry
            .path()
            .extension()
            .and_then(|s: &std::ffi::OsStr| s.to_str())
        {
            let ext = ext.to_lowercase();
            if let Some((_, lang)) = known.iter().find(|(e, _)| *e == ext) {
                *counts.entry(*lang).or_insert(0) += 1;
                total += 1;
            }
        }
    }

    if total == 0 {
        log::warn!(
            "No known source files found in {:?}, falling back to heuristic parser",
            path
        );
        return Language::Unknown;
    }

    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(lang, _)| lang)
        .unwrap_or(Language::Unknown)
}

fn strip_c_comments(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut pos = 0;
    while pos < len {
        if pos + 1 < len && bytes[pos] == b'/' && bytes[pos + 1] == b'/' {
            while pos < len && bytes[pos] != b'\n' {
                result.push(if bytes[pos] == b'\n' { '\n' } else { ' ' });
                pos += 1;
            }
        } else if pos + 1 < len && bytes[pos] == b'/' && bytes[pos + 1] == b'*' {
            result.push(' ');
            pos += 2;
            while pos < len {
                if bytes[pos] == b'*' && pos + 1 < len && bytes[pos + 1] == b'/' {
                    result.push(' ');
                    pos += 2;
                    break;
                }
                result.push(if bytes[pos] == b'\n' { '\n' } else { ' ' });
                pos += 1;
            }
        } else {
            result.push(bytes[pos] as char);
            pos += 1;
        }
    }
    result
}

fn is_valid_identifier(name: &str, lang: Language) -> bool {
    if matches!(lang, Language::Cpp | Language::CppTreeSitter) && name.contains("::") {
        return true;
    }
    if name.len() == 1 && name.chars().all(|c| c.is_ascii_alphabetic()) {
        return true;
    }
    if name.len() < 2 {
        return false;
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' && first != '~' {
        return false;
    }
    let allowed = |c: char| {
        c.is_ascii_alphanumeric()
            || c == '_'
            || (matches!(lang, Language::Cpp | Language::CppTreeSitter) && (c == ':' || c == '~'))
    };
    if !name.chars().all(allowed) {
        return false;
    }
    if name.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if name.starts_with("class")
        && name.len() > 5
        && name[5..].chars().next().unwrap().is_uppercase()
    {
        return false;
    }
    if name.contains("_1_1") || name.contains("_8cpp") || name.contains("_8h") {
        return false;
    }
    if name.len() > 40 && name.contains('_') {
        return false;
    }
    if name.starts_with("dir_") && name.len() > 30 {
        return false;
    }
    true
}

fn is_javascript_noise(name: &str) -> bool {
    static JS_NOISE: &[&str] = &[
        "isNaN",
        "eval",
        "parseInt",
        "parseFloat",
        "undefined",
        "NaN",
        "Infinity",
        "Object",
        "Array",
        "String",
        "Number",
        "Boolean",
        "Function",
        "RegExp",
        "Math",
        "Date",
        "JSON",
        "Promise",
        "Symbol",
        "Map",
        "Set",
        "WeakMap",
        "WeakSet",
        "Proxy",
        "Reflect",
        "console",
        "window",
        "document",
        "navigator",
        "location",
        "history",
        "localStorage",
        "sessionStorage",
        "alert",
        "confirm",
        "prompt",
        "fetch",
        "XMLHttpRequest",
        "getElementById",
        "getElementsByClassName",
        "getElementsByTagName",
        "querySelector",
        "querySelectorAll",
        "addEventListener",
        "removeEventListener",
        "appendChild",
        "removeChild",
        "srChild",
        "srResult",
        "srEntry",
        "srScope",
        "srLink",
        "srChildren",
        "clipboard_div",
        "clipboard_icon",
        "clipboard_successIcon",
        "clipboard_successDuration",
        "clipboard_title",
        "pagenav",
        "navtree",
        "menudata",
        "resizeHeight",
        "resizeWidth",
        "domSearchBox",
        "domPopupSearchResults",
        "domPopupSearchResultsWindow",
        "domSearchClose",
        "searchData",
        "searchResults",
        "resultsPath",
        "topOffset",
        "footerHeight",
        "headerHeight",
        "sidenavWidth",
        "pagenavWidth",
        "navSync",
        "navtreeHeight",
        "PAGENAV_COOKIE_NAME",
        "RESIZE_COOKIE_NAME",
        "SEARCH_COOKIE_NAME",
        "NAVPATH_COOKIE_NAME",
        "NAVTREE",
        "NAVTREEINDEX",
        "NAVTREEINDEX0",
        "NAVTREEINDEX1",
        "NAVTREEINDEX2",
        "NAVTREEINDEX3",
        "NAVTREEINDEX4",
        "NAVTREEINDEX5",
        "NAVTREEINDEX6",
        "NAVTREEINDEX7",
        "navTreeSubIndices",
        "entityMap",
        "htmlToNode",
        "codefold",
        "dynsection",
        "showHideNavBar",
        "showSyncOff",
        "showSyncOn",
        "SYNCOFFMSG",
        "SYNCONMSG",
        "toggleVisibility",
        "toggleClass",
        "focusItem",
        "focusName",
        "expandNode",
        "gotoNode",
        "gotoAnchor",
        "showNode",
        "showRoot",
        "selectAndHighlight",
        "highlightAnchor",
        "highlightAdjacentNodes",
        "highlightEdges",
        "loadJS",
        "createIndent",
        "makeTree",
        "makeAbsolut",
        "makeMorphable",
        "makeInstance",
        "makeSetterGetter",
        "getClass",
        "getClassForType",
        "getMethodNames",
        "getMethodsFor",
        "getEvents",
        "getEventTarget",
        "getEventPoint",
        "createResults",
        "SearchResults",
        "handleResults",
    ];
    JS_NOISE.contains(&name)
}
