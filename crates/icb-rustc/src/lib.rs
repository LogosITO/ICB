//! # ICB Rustc Backend
//!
//! This crate provides a precise analysis of Rust source code by leveraging
//! the `rustc` compiler's internal APIs.  Unlike tree‑sitter, which only
//! performs syntactic parsing, this backend resolves all semantic
//! information (traits, generics, macro expansions, etc.) and produces a
//! list of facts suitable for constructing a Code Property Graph.
//!
//! # Architecture
//!
//! - [`driver::run_rustc_analysis`] launches a `rustc` session for a given
//!   crate.
//! - During compilation, the HIR (High‑level IR) is walked by a visitor
//!   implemented in [`visitor`].
//! - The visitor records functions, methods, trait/struct/enum
//!   declarations, and call expressions as [`RawNode`] facts.
//!
//! # Feature flags
//!
//! * `nightly` – enables the `rustc_interface` linkage.  Without this flag
//!   the crate compiles as a stub that always returns an empty result.
//!   Alternatively set `RUSTC_BOOTSTRAP=1` to use the internal APIs on a
//!   nightly toolchain without the feature flag.
//!
//! # Limitations
//!
//! * The backend requires a nightly compiler or `RUSTC_BOOTSTRAP=1`.
//! * Procedural macros are not expanded by the visitor; only the expanded
//!   code is visible.
//! * The analysis is per‑crate; multi‑crate projects need to be handled at
//!   the workspace level (see [`icb_graph::builder`]).

pub mod driver;
pub mod visitor;

use anyhow::Result;
use icb_parser::facts::RawNode;
use std::path::Path;

/// Run the `rustc`‑based analysis on a single Rust crate.
///
/// If the `nightly` feature is not active, this function returns an empty
/// result immediately and logs a warning.
///
/// # Arguments
///
/// * `crate_root` – path to the root file of the crate (e.g., `src/main.rs`
///   or `src/lib.rs`).
/// * `args` – additional compiler arguments (e.g., `["--edition", "2021"]`).
///
/// # Errors
///
/// Returns an error if the `rustc` session cannot be initialised or if the
/// visitor encounters a fatal condition.
pub fn parse_rust_crate(_crate_root: &Path, _args: &[String]) -> Result<Vec<RawNode>> {
    #[cfg(feature = "nightly")]
    {
        driver::run_rustc_analysis(_crate_root, _args)
    }
    #[cfg(not(feature = "nightly"))]
    {
        log::warn!("icb-rustc backend requires the `nightly` feature (or RUSTC_BOOTSTRAP=1)");
        Ok(vec![])
    }
}