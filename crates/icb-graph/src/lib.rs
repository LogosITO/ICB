//! # icb-graph
//!
//! Code Property Graph engine.
//!
//! This crate builds and queries the Code Property Graph (CPG). It consumes
//! [`RawNode`] facts from `icb-parser`, deduplicates symbols using their
//! unique identifiers, and provides query APIs for tasks like finding all
//! functions or call sites.
//!
//! The graph is stored as a [`petgraph::stable_graph::StableGraph`] with
//! node and edge types defined in the [`graph`] module.
//!
//! # Workflow
//!
//! 1. Create a [`builder::GraphBuilder`].
//! 2. Call [`builder::GraphBuilder::ingest_file_facts`] with the facts from
//!    one or more files.
//! 3. Access the [`graph::CodePropertyGraph`] via the builder's `cpg` field.
//! 4. Use [`query`] functions to extract information.

pub mod builder;
pub mod graph;
pub mod query;
pub mod visualizer;
