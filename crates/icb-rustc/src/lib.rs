#![cfg_attr(feature = "nightly", feature(rustc_private))]

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
//! The entry point is [`parse_rust_crate`].  When the `nightly` feature is
//! enabled it calls into [`driver::run_analysis`] which starts a `rustc`
//! session, obtains the HIR, and invokes the HIR visitor implemented in
//! [`visitor`].  The visitor records functions, methods, trait/struct/enum
//! declarations, impl blocks, and call expressions as [`RawNode`] facts.
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

#[cfg(feature = "nightly")]
extern crate rustc_interface;

#[cfg(feature = "nightly")]
extern crate rustc_driver;

#[cfg(feature = "nightly")]
extern crate rustc_session;

#[cfg(feature = "nightly")]
extern crate rustc_hir;

#[cfg(feature = "nightly")]
extern crate rustc_middle;

#[cfg(feature = "nightly")]
extern crate rustc_span;

#[cfg(feature = "nightly")]
pub mod driver;

#[cfg(feature = "nightly")]
pub mod visitor;

use anyhow::Result;
use icb_parser::facts::RawNode;
use std::path::Path;

pub fn parse_rust_crate(_crate_root: &Path, _args: &[String]) -> Result<Vec<RawNode>> {
    #[cfg(feature = "nightly")]
    {
        driver::run_analysis(_crate_root, _args)
    }
    #[cfg(not(feature = "nightly"))]
    {
        log::warn!("icb-rustc backend requires the `nightly` feature (or RUSTC_BOOTSTRAP=1)");
        Ok(vec![])
    }
}
