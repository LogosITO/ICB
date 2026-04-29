use crate::facts::RawNode;
use crate::lang::python::parse_python;
use icb_common::{IcbError, Language};
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// Central entry point for parsing source code.
///
/// Supports single files and whole directories (parallel parsing).
/// Currently Python is supported out of the box.
/// For C/C++ support use the `icb-clang` crate directly
/// or via the CLI subcommand.
#[derive(Default)]
pub struct ParserManager;

impl ParserManager {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single source file into a flat list of facts.
    pub fn parse_file(&self, lang: Language, source: &str) -> Result<Vec<RawNode>, IcbError> {
        match lang {
            Language::Python => parse_python(source),
            _ => Err(IcbError::Parse(format!("unsupported language {:?}", lang))),
        }
    }

    /// Parse all source files inside a directory (recursively).
    ///
    /// Returns a vector of `(relative_path, Vec<RawNode>)` for each
    /// successfully parsed file. Files that fail to parse are skipped with
    /// a warning.
    ///
    /// # Supported extensions
    /// - Python: `.py`
    /// - C/C++: `.cpp`, `.cxx`, `.c`, `.h`, `.hpp` (via Clang, if `icb-clang` is used separately)
    pub fn parse_directory(
        &self,
        lang: Language,
        root: &Path,
    ) -> Result<Vec<(String, Vec<RawNode>)>, IcbError> {
        let mut files = Vec::new();
        collect_files(root, &mut files, lang)?;

        let results: Vec<_> = files
            .into_par_iter()
            .filter_map(|path| {
                let content = fs::read_to_string(&path).ok()?;
                match self.parse_file(lang, &content) {
                    Ok(facts) => {
                        let relative = path.strip_prefix(root).unwrap_or(&path);
                        Some((relative.display().to_string(), facts))
                    }
                    Err(e) => {
                        log::warn!("Skipping {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();
        Ok(results)
    }
}

/// Recursively collect source files with the appropriate extension for the language.
fn collect_files(
    dir: &Path,
    files: &mut Vec<std::path::PathBuf>,
    lang: Language,
) -> Result<(), IcbError> {
    let extensions: &[&str] = match lang {
        Language::Python => &["py"],
        Language::Rust => &["rs"],
        Language::JavaScript => &["js"],
        Language::Cpp => &["cpp", "cxx", "c", "h", "hpp"],
    };
    for entry in fs::read_dir(dir).map_err(IcbError::Io)? {
        let entry = entry.map_err(IcbError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, files, lang)?;
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if extensions.contains(&ext) {
                files.push(path);
            }
        }
    }
    Ok(())
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
            IcbError::Parse(msg) => assert!(msg.contains("unsupported")),
            _ => panic!("Expected Parse error"),
        }
    }
}
