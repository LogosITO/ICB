//! # icb-parser
//!
//! Language frontends that turn source code into a stream of [`RawNode`]s.
//!
//! The parser crate is the first stage of the ICB pipeline. It reads source
//! files, invokes language-specific parsers, and produces a flat list of
//! facts that the graph engine can consume.
//!
//! # Architecture
//!
//! * [`manager::ParserManager`] selects the right parser for a given
//!   [`Language`].
//! * Each language lives in its own module: [`lang`] for Python (and future
//!   languages), [`cpp_tree_sitter`] for C/C++ via tree‑sitter.
//! * Output is a [`facts::RawNode`] vector that represents AST nodes, calls,
//!   references, etc.
//!
//! # Quick example
//!
//! ```rust
//! use icb_parser::manager::ParserManager;
//! use icb_common::Language;
//!
//! let manager = ParserManager::new();
//! let facts = manager.parse_file(Language::CppTreeSitter, "void f() {}")
//!     .unwrap();
//! assert_eq!(facts.len(), 1);
//! assert_eq!(facts[0].kind, icb_common::NodeKind::Function);
//! ```

pub mod cpp_tree_sitter;
pub mod facts;
pub mod lang;
pub mod manager; // ← новая строка
