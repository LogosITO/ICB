//! # icb-parser
//!
//! Language frontends that turn source code into a stream of [`RawNode`]s.
//!
//! The parser crate is the first stage of the ICB pipeline. It reads source
//! files, invokes language-specific parsers (currently Python via
//! tree-sitter), and produces a flat list of facts that the graph engine can
//! consume.
//!
//! # Architecture
//!
//! - [`manager::ParserManager`] selects the right parser for a given
//!   [`Language`].
//! - Each language lives in its own module under [`lang`].
//! - Output is a [`facts::RawNode`] vector that represents AST nodes, calls,
//!   references, etc.

pub mod facts;
pub mod lang;
pub mod manager;
