//! Universal parser manager for ICB.
//!
//! Provides a single entry point for parsing source code in multiple
//! languages.  The manager routes requests to the appropriate backend:
//!
//! * [`Language::Python`] → [`crate::lang::python::parse_python`],
//! * [`Language::CppTreeSitter`] → [`crate::cpp_tree_sitter::parse_cpp_file`],
//! * [`Language::Cpp`] – Clang (handled externally via `icb-clang`; the
//!   manager returns an error if called directly, prompting the caller to
//!   use the Clang path).
//! * All other languages (including [`Language::Unknown`]) → universal
//!   heuristic parser.
//!
//! # Directory parsing
//!
//! When given a directory, the manager recursively discovers files with
//! language‑specific extensions and parses each one, returning a vector
//! of `(relative_path, Vec<RawNode>)`.

use crate::facts::RawNode;
use icb_common::{IcbError, Language};
use std::path::Path;
use walkdir::WalkDir;

pub struct ParserManager;

impl Default for ParserManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserManager {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single source file.
    ///
    /// # Errors
    ///
    /// Returns [`IcbError::Parse`] if the language is not supported or the
    /// source cannot be parsed.
    pub fn parse_file(&self, lang: Language, source: &str) -> Result<Vec<RawNode>, IcbError> {
        match lang {
            Language::Python => crate::lang::python::parse_python(source),
            Language::CppTreeSitter => crate::cpp_tree_sitter::parse_cpp_file(source),
            Language::Cpp => Err(IcbError::UnsupportedLanguage(
                "Cpp backend requires the `icb-clang` crate – use `CppTreeSitter` instead or call Clang directly".into(),
            )),
            Language::Unknown => crate::heuristic_parser::parse_universal(source, ""),
            _ => crate::heuristic_parser::parse_universal(source, ""),
        }
    }

    /// Recursively discover and parse files under `root` for the given
    /// language.
    ///
    /// The language determines both the file extensions and the parser.
    ///
    /// # Errors
    ///
    /// Returns [`IcbError::Io`] if directory traversal fails, or
    /// [`IcbError::Parse`] for the first file that fails to parse.
    pub fn parse_directory(
        &self,
        lang: Language,
        root: &Path,
    ) -> Result<Vec<(String, Vec<RawNode>)>, IcbError> {
        let extensions = extensions_for_language(lang);
        let mut files = Vec::new();
        for entry in WalkDir::new(root).follow_links(false) {
            let entry = entry.map_err(|e| IcbError::Parse(e.to_string()))?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if extensions.is_empty() || extensions.contains(&ext.to_lowercase().as_str()) {
                        files.push(entry.path().to_path_buf());
                    }
                } else if extensions.is_empty() {
                    files.push(entry.path().to_path_buf());
                }
            }
        }

        let relative_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let mut results = Vec::new();
        for file_path in files {
            let source = std::fs::read_to_string(&file_path).map_err(IcbError::Io)?;
            let facts = self.parse_file(lang, &source)?;
            let rel = file_path
                .strip_prefix(&relative_root)
                .unwrap_or(&file_path)
                .display()
                .to_string();
            results.push((rel, facts));
        }
        Ok(results)
    }
}

fn extensions_for_language(lang: Language) -> Vec<&'static str> {
    match lang {
        Language::Python => vec!["py"],
        Language::Cpp | Language::CppTreeSitter => vec!["c", "cpp", "cc", "cxx", "h", "hpp"],
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
