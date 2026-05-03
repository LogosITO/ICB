//! Common types shared across all ICB crates.
//!
//! # Language support
//!
//! [`Language`] enumerates all source languages that ICB can analyse.
//! For C++ two backends are available:
//!
//! * [`Cpp`] – full Clang parser (requires LLVM installation).
//! * [`CppTreeSitter`] – lightweight tree‑sitter‑cpp parser, no external
//!   dependencies.
//!
//! The caller chooses the variant; the rest of the system is agnostic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    Cpp,
    CppTreeSitter,
    Rust,
    JavaScript,
    Go,
    Java,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Scala,
    CSharp,
    Lua,
    R,
    Bash,
    Perl,
    Tcl,
    Dart,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Function,
    Class,
    Variable,
    Parameter,
    CallSite,
    Namespace,
    Enum,
}

#[derive(Debug, thiserror::Error)]
pub enum IcbError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Graph error: {0}")]
    Graph(String),
    #[error("Serialisation error: {0}")]
    Serialization(String),
}
