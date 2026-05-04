//! Language-specific parser implementations.
//!
//! Each module in this folder implements a parser for a single language.
//! They translate the native CST/AST into [`RawNode`] vectors.

pub mod go;
pub mod python;
pub mod ruby;
