//! Universal high-performance parser manager.
//!
//! # Overview
//!
//! Provides a unified, fault‑tolerant entry point for parsing source code
//! across multiple programming languages.
//!
//! # Design Goals
//!
//! * Zero‑panic parsing pipeline
//! * Deterministic behaviour
//! * Efficient large‑scale directory traversal
//! * Minimal memory overhead
//! * Extensible language backends
//!
//! # Features
//!
//! * Multi‑language dispatch
//! * Extension‑based file filtering
//! * Graceful failure handling (I/O errors, unsupported languages)
//! * Relative path normalisation
//! * Batched file processing
//!
//! # Execution Model
//!
//! 1. Discover files
//! 2. Filter by extension
//! 3. Read sources
//! 4. Dispatch parser
//! 5. Collect facts
//!
//! # Safety Guarantees
//!
//! * No `unwrap()`
//! * No panic propagation
//! * I/O errors isolated per file (skipped silently)
//! * Parse errors isolated per file (skipped silently)
//!
//! # Output
//!
//! Returns a vector of `(relative_path, Vec<RawNode>)`.

use crate::facts::RawNode;
use icb_common::{IcbError, Language};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Stateless parser manager.
#[derive(Default)]
pub struct ParserManager;

impl ParserManager {
    /// Create a new parser manager.
    pub fn new() -> Self {
        Self
    }

    /// Parse a single source file using the most appropriate backend for the
    /// given language.
    ///
    /// # Arguments
    ///
    /// * `lang` – the programming language of the source.
    /// * `source` – the raw source code as a UTF‑8 string.
    ///
    /// # Errors
    ///
    /// Returns [`IcbError::Parse`] if the specialised parser fails.
    /// Unknown / unsupported languages are handled by the universal
    /// heuristic parser and never produce an error.
    pub fn parse_file(&self, lang: Language, source: &str) -> Result<Vec<RawNode>, IcbError> {
        match lang {
            Language::Python => crate::lang::python::parse_python(source),
            Language::CppTreeSitter => crate::cpp_tree_sitter::parse_cpp_file(source),
            Language::Go => crate::lang::go::parse_go(source),
            Language::Ruby => crate::lang::ruby::parse_ruby(source),

            Language::JavaScript | Language::Rust | Language::Unknown => {
                Ok(crate::heuristic_parser::parse_universal(source, ""))
            }

            Language::Cpp => Ok(crate::heuristic_parser::parse_universal(source, "")),

            _ => Ok(crate::heuristic_parser::parse_universal(source, "")),
        }
    }

    /// Recursively discover and parse files under `root` for the given
    /// language.
    ///
    /// The language determines both the file extensions and the parser
    /// backend.  Files that cannot be read as UTF‑8 or that cause a parser
    /// error are silently skipped.
    ///
    /// # Arguments
    ///
    /// * `lang` – the programming language to use for parsing.
    /// * `root` – the root directory to walk.
    ///
    /// # Errors
    ///
    /// Returns [`IcbError::Parse`] if directory traversal fails (e.g.
    /// permission denied).  Individual file failures are not propagated.
    pub fn parse_directory(
        &self,
        lang: Language,
        root: &Path,
    ) -> Result<Vec<(String, Vec<RawNode>)>, IcbError> {
        let files = discover_files(root, lang)?;
        let base = normalize_root(root);
        let mut results = Vec::with_capacity(files.len());

        for path in files {
            match process_file(self, lang, &path, &base) {
                Ok(Some(entry)) => results.push(entry),
                Ok(None) | Err(_) => continue,
            }
        }

        Ok(results)
    }
}

// ---- Internal helpers ---------------------------------------------------

/// Walk a directory and collect every file whose extension matches the
/// language filter.
fn discover_files(root: &Path, lang: Language) -> Result<Vec<PathBuf>, IcbError> {
    let extensions = extensions_for_language(lang);
    let mut out = Vec::new();

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => return Err(IcbError::Parse(e.to_string())),
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if should_include(path, &extensions) {
            out.push(path.to_path_buf());
        }
    }

    Ok(out)
}

/// Read a single file, parse it, and return `None` on any failure.
fn process_file(
    manager: &ParserManager,
    lang: Language,
    path: &Path,
    base: &Path,
) -> Result<Option<(String, Vec<RawNode>)>, IcbError> {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    let facts = match manager.parse_file(lang, &source) {
        Ok(f) => f,
        Err(_) => return Ok(None),
    };
    if facts.is_empty() {
        return Ok(None);
    }
    let rel = relative_path(path, base);
    Ok(Some((rel, facts)))
}

/// Return the canonical form of `root`, or the original if it fails.
fn normalize_root(root: &Path) -> PathBuf {
    root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
}

/// Compute a relative path from `base` to `path`.
fn relative_path(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

/// Check whether `path` should be included based on the allowed extensions.
///
/// When `exts` is empty, **all** files are included (universal mode).
fn should_include(path: &Path, exts: &[&str]) -> bool {
    if exts.is_empty() {
        return true;
    }
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => {
            let ext = ext.to_lowercase();
            exts.iter().any(|e| *e == ext)
        }
        None => false,
    }
}

/// Return the list of file extensions accepted for a given language.
fn extensions_for_language(lang: Language) -> Vec<&'static str> {
    match lang {
        Language::Python => vec!["py"],
        Language::Cpp | Language::CppTreeSitter => vec![
            "c", "cpp", "cc", "cxx", "h", "hpp", "hxx", "hh", "inl", "inc",
        ],
        Language::Rust => vec!["rs"],
        Language::JavaScript => vec!["js", "jsx", "ts", "tsx"],
        Language::Go => vec!["go"],
        Language::Java => vec!["java"],
        Language::Ruby => vec!["rb"],
        Language::Php => vec!["php"],
        Language::Swift => vec!["swift"],
        Language::Kotlin => vec!["kt", "kts"],
        Language::Scala => vec!["scala"],
        Language::CSharp => vec!["cs"],
        Language::Lua => vec!["lua"],
        Language::R => vec!["r"],
        Language::Bash => vec!["sh", "bash"],
        Language::Perl => vec!["pl", "pm"],
        Language::Tcl => vec!["tcl"],
        Language::Dart => vec!["dart"],
        Language::Unknown => vec![],
    }
}
