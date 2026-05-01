//! Project‑level traversal of C/C++ source trees.
//!
//! # Entry points
//!
//! * [`parse_project`] – processes every translation unit listed in a
//!   [`compile_commands.json`](https://clang.llvm.org/docs/JSONCompilationDatabase.html)
//!   compilation database.
//! * [`parse_directory`] – recursively discovers C/C++ files under a root
//!   directory and parses each one with uniform compiler flags.
//!
//! Both functions distribute work across available CPU cores via
//! [`rayon::par_iter`] when `parallel` is `true`.
//!
//! # File filtering
//!
//! Only files whose extension matches one of `c`, `cpp`, `cc`, `cxx`, `h`,
//! `hpp` are considered.  Symbolic links are *not* followed to avoid
//! infinite loops on recursive directory structures.
//!
//! # Error handling
//!
//! The first file that fails to parse aborts the entire operation with an
//! [`IcbError`].  Partial results are discarded.
//!
//! # Memory usage
//!
//! Each translation unit’s facts are collected independently and then
//! returned as a flat vector.  Rayon’s work‑stealing scheduler ensures that
//! at most `num_cpus` TUs are resident in memory at any given time.
//!
//! # Example
//!
//! ```rust,no_run
//! use std::path::Path;
//! let facts = icb_clang::project::parse_directory(
//!     Path::new("src"),
//!     &["-std=c++17".into()],
//!     true,
//!     None,
//!     false,
//! ).unwrap();
//! ```

use icb_common::IcbError;
use icb_parser::facts::RawNode;
use rayon::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::parser::parse_cpp_file;

/// A single entry in a Clang compilation database.
///
/// Deserialised from `compile_commands.json`; the schema follows the
/// [Clang JSON Compilation Database Format
/// Specification](https://clang.llvm.org/docs/JSONCompilationDatabase.html).
#[derive(Debug, Deserialize)]
struct CompileCommandEntry {
    /// The main source file processed by this compilation step.
    file: String,
    /// The full compiler command line as a single string (optional).
    #[serde(default)]
    command: Option<String>,
    /// The compiler command line split into an argument list (optional).
    #[serde(default)]
    arguments: Option<Vec<String>>,
}

/// Parse every source file listed in a compilation database.
///
/// Each entry is processed independently; results are collected in the order
/// they complete.
///
/// # Arguments
///
/// * `compile_commands` – Path to `compile_commands.json`.
/// * `base_dir` – Base directory for resolving relative file paths.
/// * `parallel` – Distribute work across threads if `true`.
/// * `allow_system` – Forwarded to [`parse_cpp_file`].
///
/// # Errors
///
/// Returns [`IcbError::Io`] if the database cannot be read, or
/// [`IcbError::Parse`] for the first file that fails.
pub fn parse_project(
    compile_commands: &Path,
    base_dir: &Path,
    parallel: bool,
    allow_system: bool,
) -> Result<Vec<(String, Vec<RawNode>)>, IcbError> {
    let data = fs::read_to_string(compile_commands).map_err(IcbError::Io)?;
    let entries: Vec<CompileCommandEntry> =
        serde_json::from_str(&data).map_err(|e| IcbError::Parse(e.to_string()))?;

    let process = |entry: CompileCommandEntry| -> Result<(String, Vec<RawNode>), IcbError> {
        let file_path = resolve_file_path(&entry.file, base_dir);
        let source = fs::read_to_string(&file_path).map_err(|e| {
            IcbError::Io(std::io::Error::new(
                e.kind(),
                format!("failed to read {}: {}", file_path.display(), e),
            ))
        })?;
        let args = extract_args(&entry);
        let facts = parse_cpp_file(
            &source,
            &args,
            Some(file_path.to_str().unwrap()),
            allow_system,
        )?;
        Ok((file_path.to_string_lossy().into_owned(), facts))
    };

    if parallel {
        entries
            .into_par_iter()
            .map(process)
            .collect::<Result<Vec<_>, _>>()
    } else {
        entries
            .into_iter()
            .map(process)
            .collect::<Result<Vec<_>, _>>()
    }
}

/// Recursively discover C/C++ files under `root` and parse each one.
///
/// Only files with an extension in `{c, cpp, cc, cxx, h, hpp}` are
/// processed.  Symlinks are ignored to prevent infinite recursion.
///
/// # Arguments
///
/// * `root` – Root directory for the walk.
/// * `args` – Clang command‑line arguments shared by all files.
/// * `parallel` – Distribute work across threads if `true`.
/// * `max_depth` – Maximum directory depth (`None` for unlimited).
/// * `allow_system` – Forwarded to [`parse_cpp_file`].
///
/// # Errors
///
/// Returns [`IcbError::Io`] if the directory walk fails, or
/// [`IcbError::Parse`] for the first file that fails.
pub fn parse_directory(
    root: &Path,
    args: &[String],
    parallel: bool,
    max_depth: Option<usize>,
    allow_system: bool,
) -> Result<Vec<(String, Vec<RawNode>)>, IcbError> {
    let mut files = Vec::new();
    collect_cpp_files(root, &mut files, max_depth)?;

    let process = |path: PathBuf| -> Result<(String, Vec<RawNode>), IcbError> {
        let source = fs::read_to_string(&path).map_err(IcbError::Io)?;
        let facts = parse_cpp_file(&source, args, Some(path.to_str().unwrap()), allow_system)?;
        let rel = path.strip_prefix(root).unwrap_or(&path);
        Ok((rel.display().to_string(), facts))
    };

    if parallel {
        files
            .into_par_iter()
            .map(process)
            .collect::<Result<Vec<_>, _>>()
    } else {
        files
            .into_iter()
            .map(process)
            .collect::<Result<Vec<_>, _>>()
    }
}

/// Resolve a file path relative to `base_dir`.
///
/// Absolute paths are returned unchanged.  Leading/trailing whitespace is
/// trimmed from `file` before resolution.
fn resolve_file_path(file: &str, base: &Path) -> PathBuf {
    let path = Path::new(file.trim());
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

/// Extract compiler arguments from a compilation database entry.
///
/// Prefers the `arguments` field if present; otherwise splits `command` on
/// whitespace.  Returns an empty vector if neither field is set.
fn extract_args(entry: &CompileCommandEntry) -> Vec<String> {
    if let Some(ref arguments) = entry.arguments {
        return arguments.clone();
    }
    if let Some(ref command) = entry.command {
        return command.split_whitespace().map(|s| s.to_string()).collect();
    }
    Vec::new()
}

/// Walk the directory tree and collect C/C++ source files.
///
/// Symlinks are not followed, and the optional `max_depth` limits recursion.
/// Walk the directory tree and collect C/C++ source files.
///
/// Symlinks are not followed, and the optional `max_depth` limits recursion.
/// File extensions are matched case‑insensitively.
fn collect_cpp_files(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    max_depth: Option<usize>,
) -> Result<(), IcbError> {
    let cpp_extensions: &[&str] = &["c", "cpp", "cc", "cxx", "h", "hpp"];
    for entry in WalkDir::new(dir).follow_links(false) {
        let entry = entry.map_err(|e| IcbError::Parse(e.to_string()))?;
        if let Some(max) = max_depth {
            if entry.depth() > max {
                continue;
            }
        }
        if !entry.file_type().is_file() {
            continue;
        }
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            if cpp_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                files.push(entry.path().to_path_buf());
            }
        }
    }
    Ok(())
}

#[doc(hidden)]
pub fn collect_cpp_files_for_preview(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    max_depth: Option<usize>,
) -> Result<(), IcbError> {
    collect_cpp_files(dir, files, max_depth)
}
