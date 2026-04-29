//! # icb-clang
//!
//! Clang‑based C/C++ frontend for ICB.
//!
//! Provides parsing of single files or entire projects described by
//! `compile_commands.json`. The parser returns a stream of [`RawNode`]
//! facts that the rest of the ICB pipeline can consume.
//!
//! # Example (single file)
//!
//! ```rust,no_run
//! use icb_clang::parse_cpp_file;
//! let facts = parse_cpp_file("int main() { return 0; }", &[]).unwrap();
//! assert!(!facts.is_empty());
//! ```
//!
//! # Example (project via compile_commands.json)
//!
//! ```rust,no_run
//! use icb_clang::parse_project;
//! let all_files = parse_project(
//!     "path/to/compile_commands.json",
//!     true,   // parallel
//! ).unwrap();
//! ```

pub mod parser;
pub mod project;
