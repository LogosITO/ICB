use icb_common::IcbError;
use icb_parser::facts::RawNode;
use rayon::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::parser::parse_cpp_file;

/// A single entry in a `compile_commands.json` database.
#[derive(Debug, Deserialize)]
struct CompileCommandEntry {
    file: String,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    arguments: Option<Vec<String>>,
}

/// Parse an entire C/C++ project described by `compile_commands.json`.
///
/// `base_dir` is used to resolve relative paths in the `file` fields.
/// If `parallel` is `true`, translation units are processed in parallel
/// using rayon.
///
/// # Errors
///
/// Returns [`IcbError::Parse`] if the compilation database cannot be read
/// or any translation unit fails to parse.
pub fn parse_project(
    compile_commands: &Path,
    base_dir: &Path,
    parallel: bool,
) -> Result<Vec<(String, Vec<RawNode>)>, IcbError> {
    let data = fs::read_to_string(compile_commands).map_err(IcbError::Io)?;
    let entries: Vec<CompileCommandEntry> =
        serde_json::from_str(&data).map_err(|e| IcbError::Parse(e.to_string()))?;

    let process_entry = |entry: CompileCommandEntry| -> Result<(String, Vec<RawNode>), IcbError> {
        let file_path = resolve_file_path(&entry.file, base_dir);
        let source = fs::read_to_string(&file_path).map_err(|e| {
            IcbError::Io(std::io::Error::new(
                e.kind(),
                format!("failed to read {}: {}", file_path.display(), e),
            ))
        })?;
        let args = extract_args(&entry);
        let rel_name = file_path
            .strip_prefix(base_dir)
            .unwrap_or(&file_path)
            .to_str()
            .unwrap_or("unknown");
        let facts = parse_cpp_file(&source, &args, Some(rel_name))?;
        Ok((file_path.to_string_lossy().into_owned(), facts))
    };

    if parallel {
        entries
            .into_par_iter()
            .map(process_entry)
            .collect::<Result<Vec<_>, _>>()
    } else {
        entries
            .into_iter()
            .map(process_entry)
            .collect::<Result<Vec<_>, _>>()
    }
}

fn resolve_file_path(file: &str, base: &Path) -> PathBuf {
    let path = Path::new(file.trim());
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn extract_args(entry: &CompileCommandEntry) -> Vec<String> {
    if let Some(ref arguments) = entry.arguments {
        return arguments.clone();
    }
    if let Some(ref command) = entry.command {
        return command.split_whitespace().map(|s| s.to_string()).collect();
    }
    Vec::new()
}

/// Parse all C/C++ files in `root` (recursive) and return facts.
///
/// `args` are passed to each translation unit. If `parallel` is `true`,
/// files are processed in parallel.
pub fn parse_directory(
    root: &Path,
    args: &[String],
    parallel: bool,
) -> Result<Vec<(String, Vec<RawNode>)>, IcbError> {
    let mut files = Vec::new();
    collect_cpp_files(root, &mut files)?;

    let process_file = |path: PathBuf| -> Result<(String, Vec<RawNode>), IcbError> {
        let source = fs::read_to_string(&path).map_err(IcbError::Io)?;
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let facts = parse_cpp_file(&source, args, Some(rel.to_str().unwrap_or("unknown")))?;
        Ok((rel.display().to_string(), facts))
    };

    if parallel {
        files
            .into_par_iter()
            .map(process_file)
            .collect::<Result<Vec<_>, _>>()
    } else {
        files
            .into_iter()
            .map(process_file)
            .collect::<Result<Vec<_>, _>>()
    }
}

fn collect_cpp_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), IcbError> {
    let cpp_extensions = ["c", "cpp", "cc", "cxx", "h", "hpp"];
    for entry in fs::read_dir(dir).map_err(IcbError::Io)? {
        let entry = entry.map_err(IcbError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            collect_cpp_files(&path, files)?;
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if cpp_extensions.contains(&ext) {
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
    fn test_parse_directory() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.cpp");
        std::fs::write(&file_path, "int x = 42;").unwrap();

        let results = parse_directory(dir.path(), &["-std=c++17".to_string()], false).unwrap();
        assert_eq!(results.len(), 1);
        let facts = &results[0].1;
        assert!(facts
            .iter()
            .any(|n| n.kind == icb_common::NodeKind::Variable));
    }

    #[test]
    fn test_parse_project() {
        let dir = tempfile::tempdir().unwrap();
        let cpp_file = dir.path().join("test.cpp");
        std::fs::write(&cpp_file, "int main() { return 0; }").unwrap();

        let cdb_path = dir.path().join("compile_commands.json");
        let cdb = serde_json::json!([
            {
                "file": "test.cpp",
                "arguments": ["clang++", "-std=c++17", "test.cpp"],
            }
        ]);
        std::fs::write(&cdb_path, cdb.to_string()).unwrap();

        let results = parse_project(&cdb_path, dir.path(), false).unwrap();
        assert_eq!(results.len(), 1);
        let facts = &results[0].1;
        assert!(facts
            .iter()
            .any(|n| n.kind == icb_common::NodeKind::Function));
    }
}
