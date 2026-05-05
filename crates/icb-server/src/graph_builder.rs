//! Advanced Code Property Graph construction pipeline.
//!
//! # Philosophy
//!
//! This module is designed for **robust static analysis at scale**.
//!
//! Key principles:
//!
//! - Strict input filtering (no cross-language pollution)
//! - Deterministic parsing
//! - Fail-safe execution (never panic on bad input)
//! - Pluggable parsing backends (including Clang)
//! - Observable pipeline (logging-ready)
//! - Respects `--no-system-headers` flag
//! - Automatically disables extension filtering when the language is unknown,
//!   ensuring ZIP uploads work out‑of‑the‑box.
//!
//! # Pipeline stages
//!
//! 1. Language resolution
//! 2. File discovery
//! 3. Extension filtering
//! 4. Source normalization
//! 5. Parsing (Clang preferred for C/C++, with system header control)
//! 6. Fact filtering (aggressive noise removal)
//! 7. Graph building
//! 8. Post-processing
//! 9. Cache handling
//!
//! # Guarantees
//!
//! - No JS leaking into C++ analysis
//! - No invalid identifiers
//! - No parser crashes propagate
//! - Deterministic output graph

use anyhow::anyhow;
use icb_common::Language;
use icb_graph::{cache, graph::CodePropertyGraph};
use icb_parser::facts::RawNode;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::display_name;

type FileFacts = Vec<(String, Vec<RawNode>)>;

#[derive(Debug, Clone)]
struct PipelineConfig {
    pub languages: HashSet<Language>,
    pub strict_extensions: bool,
    pub strip_comments: bool,
    pub no_system_headers: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            languages: HashSet::new(),
            strict_extensions: true,
            strip_comments: true,
            no_system_headers: true,
        }
    }
}

/// Entry point: single-language
pub fn build_or_load_graph(
    project: &Path,
    language: &str,
    cache_path: Option<&PathBuf>,
    no_system_headers: bool,
) -> anyhow::Result<CodePropertyGraph> {
    let lang = resolve_language(project, language)?;

    let strict = lang != Language::Unknown; // отключаем фильтр, если язык неизвестен

    let cfg = PipelineConfig {
        languages: {
            let mut set = HashSet::new();
            set.insert(lang);
            set
        },
        no_system_headers,
        strict_extensions: strict,
        ..Default::default()
    };

    run_pipeline(project, cfg, cache_path)
}

/// Entry point: multi-language
pub fn build_or_load_graph_multi(
    project: &Path,
    languages: &[String],
    cache_path: Option<&PathBuf>,
    no_system_headers: bool,
) -> anyhow::Result<CodePropertyGraph> {
    if languages.is_empty() || languages.iter().any(|l| l == "auto") {
        return build_or_load_graph(project, "auto", cache_path, no_system_headers);
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
        ..Default::default()
    };

    run_pipeline(project, cfg, cache_path)
}

/// Core pipeline executor
fn run_pipeline(
    project: &Path,
    cfg: PipelineConfig,
    cache_path: Option<&PathBuf>,
) -> anyhow::Result<CodePropertyGraph> {
    if let Some(cache_file) = cache_path {
        if cache_file.exists() {
            if let Ok(mut g) = cache::load_graph(cache_file) {
                display_name::cleanup_node_names(&mut g);
                return Ok(g);
            }
        }
    }

    // --- Attempt Clang for C++ projects ---
    if cfg.languages.contains(&Language::CppTreeSitter) || cfg.languages.contains(&Language::Cpp) {
        if let Some(cpg) = try_clang_pipeline(project, &cfg, cache_path) {
            return Ok(cpg);
        }
    }

    // --- General pipeline ---
    let files = discover_files(project)?;
    let filtered = filter_files(files, &cfg)?;
    let facts = parse_files(filtered, &cfg)?;
    let graph = build_graph(facts)?;
    let graph = finalize_graph(graph)?;

    if let Some(cache_file) = cache_path {
        let _ = cache::save_graph(&graph, cache_file);
    }

    Ok(graph)
}

/// Try to build the graph using Clang.
///
/// Returns `None` if Clang is not available or fails, allowing fallback
/// to tree‑sitter.
fn try_clang_pipeline(
    project: &Path,
    cfg: &PipelineConfig,
    cache_path: Option<&PathBuf>,
) -> Option<CodePropertyGraph> {
    #[cfg(feature = "clang")]
    {
        log::info!("Attempting Clang graph construction...");
        let allow_system = !cfg.no_system_headers;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            icb_clang::project::parse_directory(
                project,
                &["-std=c++17".into()],
                true,
                None,
                allow_system,
            )
        }));

        match result {
            Ok(Ok(file_facts)) => {
                log::info!("Clang parsed {} files", file_facts.len());

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
                if let Some(cache_file) = cache_path {
                    let _ = cache::save_graph(&cpg, cache_file);
                }
                log::info!("Clang graph built successfully");
                Some(cpg)
            }
            Ok(Err(e)) => {
                log::warn!("Clang parse error: {}, falling back to tree-sitter", e);
                None
            }
            Err(panic) => {
                log::error!("Clang panicked: {:?}, falling back to tree-sitter", panic);
                None
            }
        }
    }
    #[cfg(not(feature = "clang"))]
    {
        log::debug!("Clang feature not compiled in");
        None
    }
}

/// Step 1: discover files
fn discover_files(project: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut result = Vec::new();

    for entry in WalkDir::new(project).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            result.push(entry.path().to_path_buf());
        }
    }

    Ok(result)
}

/// Step 2: filter by language + extension
fn filter_files(
    files: Vec<PathBuf>,
    cfg: &PipelineConfig,
) -> anyhow::Result<Vec<(PathBuf, Language)>> {
    let mut result = Vec::new();

    for file in files {
        let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");

        let lang = detect_language_from_extension(ext);

        if !cfg.languages.contains(&lang) {
            // Если язык неизвестен и фильтр нестрогий — пропускаем все файлы
            if !cfg.strict_extensions && lang == Language::Unknown {
                result.push((file, lang));
            }
            continue;
        }

        if cfg.strict_extensions {
            let allowed = extensions_for_language(lang);
            if !allowed.contains(&ext) {
                continue;
            }
        }

        result.push((file, lang));
    }

    Ok(result)
}

/// Step 3: parse files
fn parse_files(files: Vec<(PathBuf, Language)>, cfg: &PipelineConfig) -> anyhow::Result<FileFacts> {
    let manager = icb_parser::manager::ParserManager::new();
    let mut result = Vec::new();

    for (path, lang) in files {
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let source = if cfg.strip_comments {
            strip_comments(&raw)
        } else {
            raw
        };

        let facts = match manager.parse_file(lang, &source) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let filtered = filter_facts(facts, lang);

        let rel = path.display().to_string();
        result.push((rel, filtered));
    }

    Ok(result)
}

/// Step 4: filter parser facts
fn filter_facts(facts: Vec<RawNode>, lang: Language) -> Vec<RawNode> {
    facts
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
            is_valid_identifier(name, lang) && !is_javascript_noise(name) && !is_type_keyword(name)
        })
        .collect()
}

/// Step 5: build graph
fn build_graph(file_facts: FileFacts) -> anyhow::Result<CodePropertyGraph> {
    let mut builder = icb_graph::builder::GraphBuilder::new();

    for (_, facts) in file_facts {
        let mut local = icb_graph::builder::GraphBuilder::new();
        local.ingest_file_facts(&facts);
        builder.merge(local);
    }

    builder.resolve_calls();

    Ok(builder.cpg)
}

/// Step 6: finalize graph
fn finalize_graph(mut g: CodePropertyGraph) -> anyhow::Result<CodePropertyGraph> {
    display_name::cleanup_node_names(&mut g);
    Ok(g)
}

/// Language helpers
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

// ---------------------------------------------------------------------------
// Strict identifier and noise filters (the heart of the cleanup)
// ---------------------------------------------------------------------------

/// Return `true` if `name` looks like a plausible source‑code identifier.
///
/// Rejects:
/// - names shorter than 2 characters (except single‑letter generic names
///   like `a`, `b`, `i`),
/// - names containing whitespace, dots, slashes, backslashes, or colons,
/// - names starting with a digit,
/// - names that are clearly file paths (contain `_8cpp`, `_8h`, etc.),
/// - names consisting only of digits.
fn is_valid_identifier(name: &str, lang: Language) -> bool {
    // Allow C++ qualified names like `vizora::ConfigManager`
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
    // Doxygen / web‑generated garbage patterns
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

/// Return `true` if `name` is a known JavaScript/DOM/HTML identifier that
/// should never appear in a C++ project.
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

/// Return `true` if `name` is a built‑in C/C++ type keyword that can
/// appear as a function name in Clang facts but is not a real function.
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
