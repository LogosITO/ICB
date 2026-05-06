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
//! # Incremental fact caching
//!
//! When a cache directory is provided (or the default `.icb_cache` is
//! used), the module stores extracted facts for every source file together
//! with a SHA-256 hash of the file content.  On subsequent runs, files that
//! have not changed are loaded directly from the cache, skipping the parser
//! entirely.  This can reduce the analysis time for large projects from
//! seconds to milliseconds.
//!
//! The incremental cache is **transparently used by both the Clang and
//! tree-sitter backends** – you get fast reloads regardless of the chosen
//! parser.
//!
//! # C/C++: Clang preferred, tree-sitter fallback
//!
//! For C and C++ projects the module **prefers the Clang parser** when the
//! `icb-clang` crate is available.  Clang provides exact semantic
//! analysis and never mistakes documentation, HTML, or embedded JavaScript
//! for C++ code.  If Clang cannot be loaded (e.g. LLVM not installed), the
//! pipeline automatically falls back to tree-sitter-cpp.
//!
//! # Strict file extension filtering & multi-language support
//!
//! For every known language, only a curated list of file extensions is
//! accepted.  This eliminates noise from documentation, build artefacts,
//! and web assets.  When `build_or_load_graph_multi` is called with a
//! specific list of languages, only those extensions are scanned.
//!
//! # Auto-detection of language
//!
//! When `language` is `"auto"`, the module scans the project directory
//! and picks the dominant language based on file extensions.  Unknown
//! extensions fall back to the universal heuristic parser.

use anyhow::anyhow;
use icb_common::Language;
use icb_graph::{cache, graph::CodePropertyGraph};
use icb_parser::facts::RawNode;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use walkdir::WalkDir;

use crate::display_name;
use crate::incremental_cache::IncrementalCache;

#[derive(Debug, Clone)]
struct PipelineConfig {
    pub languages: HashSet<Language>,
    pub strict_extensions: bool,
    pub strip_comments: bool,
    pub no_system_headers: bool,
    pub inc_cache_dir: Option<PathBuf>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            languages: HashSet::new(),
            strict_extensions: true,
            strip_comments: true,
            no_system_headers: true,
            inc_cache_dir: None,
        }
    }
}

pub fn build_or_load_graph(
    project: &Path,
    language: &str,
    graph_cache_path: Option<&PathBuf>,
    inc_cache_dir: Option<&PathBuf>,
    no_system_headers: bool,
) -> anyhow::Result<CodePropertyGraph> {
    let lang = resolve_language(project, language)?;
    let strict = lang != Language::Unknown;

    let cfg = PipelineConfig {
        languages: {
            let mut set = HashSet::new();
            set.insert(lang);
            set
        },
        no_system_headers,
        strict_extensions: strict,
        inc_cache_dir: inc_cache_dir.cloned(),
        ..Default::default()
    };

    run_pipeline(project, cfg, graph_cache_path)
}

pub fn build_or_load_graph_multi(
    project: &Path,
    languages: &[String],
    graph_cache_path: Option<&PathBuf>,
    inc_cache_dir: Option<&PathBuf>,
    no_system_headers: bool,
) -> anyhow::Result<CodePropertyGraph> {
    if languages.is_empty() || languages.iter().any(|l| l == "auto") {
        return build_or_load_graph(
            project,
            "auto",
            graph_cache_path,
            inc_cache_dir,
            no_system_headers,
        );
    }

    let cfg = PipelineConfig {
        languages: {
            let mut set = HashSet::new();
            for l in languages {
                if let Some(lang) = parse_language(l) {
                    set.insert(lang);
                }
            }
            set
        },
        no_system_headers,
        strict_extensions: !languages
            .iter()
            .any(|l| parse_language(l) == Some(Language::Unknown)),
        inc_cache_dir: inc_cache_dir.cloned(),
        ..Default::default()
    };

    run_pipeline(project, cfg, graph_cache_path)
}

fn run_pipeline(
    project: &Path,
    cfg: PipelineConfig,
    graph_cache_path: Option<&PathBuf>,
) -> anyhow::Result<CodePropertyGraph> {
    if let Some(cache_file) = graph_cache_path {
        if cache_file.exists() {
            if let Ok(mut g) = cache::load_graph(cache_file) {
                display_name::cleanup_node_names(&mut g);
                return Ok(g);
            }
        }
    }

    let inc_cache = cfg
        .inc_cache_dir
        .as_ref()
        .map(|dir| {
            if dir.extension().is_some() {
                let mut d = dir.clone();
                d.set_extension("");
                IncrementalCache::new(&d)
            } else {
                IncrementalCache::new(dir)
            }
        })
        .transpose()?
        .or_else(|| IncrementalCache::new(&project.join(".icb_cache")).ok());

    // FIX: removed invalid Language::Cpp
    if cfg.languages.contains(&Language::CppTreeSitter) {
        if let Some(cpg) = try_clang_pipeline(project, &cfg, graph_cache_path, inc_cache.as_ref()) {
            return Ok(cpg);
        }
    }

    let manager = Arc::new(icb_parser::manager::ParserManager::new());
    let mut facts: Vec<(String, Vec<RawNode>)> = Vec::new();

    for entry in WalkDir::new(project)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        if cfg.strict_extensions {
            let lang = detect_language_from_extension(ext);
            if !cfg.languages.contains(&lang) {
                continue;
            }
            let allowed = extensions_for_language(lang);
            if !allowed.contains(&ext) {
                continue;
            }
        }

        let rel = path
            .strip_prefix(project)
            .unwrap_or(path)
            .display()
            .to_string();

        let lang = if cfg.languages.len() == 1 {
            *cfg.languages.iter().next().unwrap()
        } else {
            detect_language_from_extension(ext)
        };

        if let Some(ref cache) = inc_cache {
            let manager = Arc::clone(&manager);
            let file_facts = cache.process_file(
                path,
                &rel,
                Box::new(move |source: &str| -> anyhow::Result<Vec<RawNode>> {
                    manager.parse_file(lang, source).map_err(|e| anyhow!(e))
                }),
            )?;
            facts.push((file_facts.relative_path, file_facts.facts));
        } else {
            let raw_source = fs::read_to_string(path).unwrap_or_default();
            let source = if cfg.strip_comments {
                strip_comments(&raw_source)
            } else {
                raw_source
            };

            let file_facts =
                match icb_parser::manager::ParserManager::new().parse_file(lang, &source) {
                    Ok(f) => f,
                    Err(_) => continue,
                };

            facts.push((rel, file_facts));
        }
    }

    let mut builder = icb_graph::builder::GraphBuilder::new();
    for (_, file_facts) in facts {
        let filtered: Vec<_> = file_facts
            .into_iter()
            .filter(|f| {
                matches!(
                    f.kind,
                    icb_common::NodeKind::Function
                        | icb_common::NodeKind::Class
                        | icb_common::NodeKind::CallSite
                )
            })
            .filter(|f| !f.name.as_deref().unwrap_or("").is_empty())
            .collect();

        let mut local = icb_graph::builder::GraphBuilder::new();
        local.ingest_file_facts(&filtered);
        builder.merge(local);
    }

    builder.resolve_calls();

    let mut cpg = builder.cpg;
    display_name::cleanup_node_names(&mut cpg);

    if let Some(cache_file) = graph_cache_path {
        let _ = cache::save_graph(&cpg, cache_file);
    }

    Ok(cpg)
}

fn try_clang_pipeline(
    project: &Path,
    cfg: &PipelineConfig,
    graph_cache_path: Option<&PathBuf>,
    inc_cache: Option<&IncrementalCache>,
) -> Option<CodePropertyGraph> {
    #[cfg(feature = "clang")]
    {
        log::info!("Attempting Clang graph construction with incremental cache...");
        let allow_system = !cfg.no_system_headers;

        let mut facts: Vec<(String, Vec<RawNode>)> = Vec::new();

        for entry in WalkDir::new(project)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

            // Only C++ extensions
            let allowed = extensions_for_language(Language::CppTreeSitter);
            if !allowed.contains(&ext) {
                continue;
            }

            let rel = path
                .strip_prefix(project)
                .unwrap_or(path)
                .display()
                .to_string();

            if let Some(cache) = inc_cache {
                let file_facts = cache
                    .process_file(
                        path,
                        &rel,
                        Box::new(move |source: &str| -> anyhow::Result<Vec<RawNode>> {
                            icb_clang::parser::parse_cpp_file(
                                source,
                                &["-std=c++17".to_string()],
                                None,
                                allow_system,
                            )
                            .map_err(|e| anyhow!(e))
                        }),
                    )
                    .ok()?;
                facts.push((file_facts.relative_path, file_facts.facts));
            } else {
                let source = std::fs::read_to_string(path).ok()?;
                let file_facts = icb_clang::parser::parse_cpp_file(
                    &source,
                    &["-std=c++17".to_string()],
                    None,
                    allow_system,
                )
                .ok()?;
                facts.push((rel, file_facts));
            }
        }

        log::info!("Clang processed {} files", facts.len());

        let mut builder = icb_graph::builder::GraphBuilder::new();
        for (_, file_facts) in facts {
            let filtered: Vec<_> = file_facts
                .into_iter()
                .filter(|f| {
                    matches!(
                        f.kind,
                        icb_common::NodeKind::Function
                            | icb_common::NodeKind::Class
                            | icb_common::NodeKind::CallSite
                    )
                })
                .filter(|f| {
                    let name = f.name.as_deref().unwrap_or("");
                    is_valid_identifier(name, Language::CppTreeSitter)
                        && !is_javascript_noise(name)
                        && !is_type_keyword(name)
                })
                .collect();
            let mut local = icb_graph::builder::GraphBuilder::new();
            local.ingest_file_facts(&filtered);
            builder.merge(local);
        }
        builder.resolve_calls();
        let mut cpg = builder.cpg;
        display_name::cleanup_node_names(&mut cpg);
        if let Some(cache_file) = graph_cache_path {
            let _ = cache::save_graph(&cpg, cache_file);
        }
        log::info!("Clang graph built successfully");
        Some(cpg)
    }
    #[cfg(not(feature = "clang"))]
    {
        log::debug!("Clang feature not compiled in");
        None
    }
}

fn resolve_language(project: &Path, input: &str) -> anyhow::Result<Language> {
    if input == "auto" {
        Ok(detect_language_from_project(project))
    } else {
        parse_language(input).ok_or_else(|| anyhow!("unknown language"))
    }
}

fn parse_language(s: &str) -> Option<Language> {
    match s {
        "cpp" | "c++" => Some(Language::CppTreeSitter),
        "python" => Some(Language::Python),
        "go" => Some(Language::Go),
        "ruby" => Some(Language::Ruby),
        "rust" => Some(Language::Rust),
        "javascript" => Some(Language::JavaScript),
        _ => None,
    }
}

fn detect_language_from_extension(ext: &str) -> Language {
    match ext {
        "cpp" | "cc" | "cxx" | "h" | "hpp" => Language::CppTreeSitter,
        "py" => Language::Python,
        "rs" => Language::Rust,
        "go" => Language::Go,
        "rb" => Language::Ruby,
        "js" | "ts" | "tsx" | "jsx" => Language::JavaScript,
        _ => Language::Unknown,
    }
}

fn detect_language_from_project(path: &Path) -> Language {
    let mut counts: HashMap<Language, usize> = HashMap::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            let lang = detect_language_from_extension(ext);
            *counts.entry(lang).or_insert(0) += 1;
        }
    }

    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(l, _)| l)
        .unwrap_or(Language::Unknown)
}

fn extensions_for_language(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::CppTreeSitter => &["cpp", "cc", "cxx", "h", "hpp"],
        Language::Python => &["py"],
        Language::Rust => &["rs"],
        Language::Go => &["go"],
        Language::Ruby => &["rb"],
        Language::JavaScript => &["js", "ts", "tsx", "jsx"],
        _ => &[],
    }
}

fn strip_comments(s: &str) -> String {
    s.replace("//", " ").replace("/*", " ").replace("*/", " ")
}

fn is_valid_identifier(name: &str, lang: Language) -> bool {
    if matches!(lang, Language::CppTreeSitter | Language::Cpp) && name.contains("::") {
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
            || (matches!(lang, Language::CppTreeSitter | Language::Cpp) && (c == ':' || c == '~'))
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

fn is_type_keyword(name: &str) -> bool {
    matches!(
        name,
        "void"
            | "int"
            | "long"
            | "short"
            | "char"
            | "float"
            | "double"
            | "signed"
            | "unsigned"
            | "bool"
            | "wchar_t"
            | "size_t"
    )
}
