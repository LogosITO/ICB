//! # icb-report
//!
//! Generates static HTML reports and diffs from the Code Property Graph.
//!
//! ## Report
//!
//! Creates a self‑contained HTML page with interactive graph, statistics,
//! and lists of cycles, dead code, and complex functions.
//!
//! ## Diff
//!
//! Compares two versions of a project and highlights added, removed, and
//! modified functions in an interactive graph.

pub mod diff;
pub mod report;
