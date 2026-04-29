//! # icb-clang
//!
//! Clang‑based C/C++ frontend for ICB.
//!
//! Translates translation units into the unified [`RawNode`] stream used by
//! the rest of the pipeline. It links against `libclang` and requires a
//! working installation of LLVM/Clang (see README for setup instructions).
//!
//! # Example
//!
//! ```rust,no_run
//! use icb_clang::parser::parse_cpp;
//! let facts = parse_cpp("#include <stdio.h>\nint main() { return 0; }").unwrap();
//! assert!(!facts.is_empty());
//! ```

pub mod parser;
