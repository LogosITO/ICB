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
//!
//! # Directory parsing
//!
//! When given a directory, the manager recursively discovers files with
//! language‑specific extensions and parses each one, returning a vector
//! of `(relative_path, Vec<RawNode>)`.

use std::path::Path;

use icb_common::{IcbError, Language};
use walkdir::WalkDir;

use crate::facts::RawNode;

/// Parser manager with no internal state.
#[derive(Default)]
pub struct ParserManager;

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
                "Cpp backend requires the `icb-clang` crate – use `CppTreeSitter` instead or call \
                 Clang directly"
                    .into(),
            )),
            other => Err(IcbError::UnsupportedLanguage(format!("{:?}", other))),
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
                    if extensions.contains(&ext.to_lowercase().as_str()) {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        let relative_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let mut results = Vec::new();
        for file_path in files {
            let source = std::fs::read_to_string(&file_path).map_err(IcbError::Io)?;
            let facts = match lang {
                Language::Cpp => {
                    return Err(IcbError::UnsupportedLanguage(
                        "Cpp backend requires Clang".into(),
                    ));
                }
                _ => self.parse_file(lang, &source)?,
            };
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

fn extensions_for_language(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::Python => &["py"],
        Language::Cpp | Language::CppTreeSitter => &["c", "cpp", "cc", "cxx", "h", "hpp"],
        Language::Rust => &["rs"],
        Language::JavaScript => &["js", "jsx", "ts", "tsx"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_language_returns_error() {
        let manager = ParserManager::new();
        let res = manager.parse_file(Language::Rust, "fn main() {}");
        assert!(res.is_err());
        match res.unwrap_err() {
            IcbError::UnsupportedLanguage(_) => {}
            other => panic!("Expected UnsupportedLanguage, got {:?}", other),
        }
    }
}
