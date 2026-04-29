//! # icb-common
//!
//! Core types and error handling shared across all ICB crates.
//!
//! This crate defines the universal vocabulary for languages, node kinds,
//! and errors. Everything in `icb-common` is designed to be serialisable,
//! lightweight, and independent of any particular parser or graph engine.

use serde::{Deserialize, Serialize};

/// Supported programming languages.
///
/// The language enum is used by the parser manager to dispatch to the
/// appropriate frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Rust source code (`.rs`).
    Rust,
    /// Python source code (`.py`).
    Python,
    /// JavaScript source code (`.js`).
    JavaScript,
    /// C/C++ source code (`.c`, `.cpp`, `.h`, etc.).
    Cpp,
}

/// Kinds of nodes that can appear in a Code Property Graph.
///
/// Node kinds abstract away language-specific AST node types and provide a
/// uniform interface for graph queries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    /// A translation unit / module.
    Module,
    /// A function definition.
    Function,
    /// A class or struct definition.
    Class,
    /// A variable or field.
    Variable,
    /// A call expression (function/method call).
    CallSite,
    /// A function parameter.
    Parameter,
    /// An import statement.
    Import,
}

/// Unified error type for all ICB operations.
///
/// Errors from parsing, graph construction, and I/O are all mapped to
/// `IcbError`, making it easy to propagate failures through the system.
#[derive(Debug, thiserror::Error)]
pub enum IcbError {
    /// An error that occurred during parsing.
    #[error("Parse error: {0}")]
    Parse(String),
    /// An error that occurred during graph building or querying.
    #[error("Graph error: {0}")]
    Graph(String),
    /// A wrapper around [`std::io::Error`].
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
