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

/// Programming language of a source file or project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    /// C/C++ via Clang (default for `cpp`).
    Cpp,
    /// C/C++ via tree-sitter-cpp (fast, portable).
    CppTreeSitter,
    Rust,
    JavaScript,
}

/// Kinds of nodes that can appear in a [`RawNode`](icb_parser::facts::RawNode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Function,
    Class,
    Variable,
    Parameter,
    CallSite,
    Namespace,
    Enum,
    // … другие варианты могут быть добавлены
}

/// Error type for the whole workspace.
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
