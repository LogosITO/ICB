//! Diff engine for comparing two Code Property Graphs.
//!
//! # Overview
//!
//! This module loads or builds two [`CodePropertyGraph`]s (typically from
//! two projects or two cached snapshots) and computes a symmetric
//! difference: which functions, classes and call-edges were added, removed
//! or stayed the same.
//!
//! # Output
//!
//! The result is a [`DiffReport`] containing two flat vectors:
//!
//! * `nodes` – every node present in at least one of the two graphs,
//!   tagged with [`Status`].
//! * `edges` – every unique `(caller, callee, kind)` tuple, also tagged
//!   with its status.
//!
//! # Matching strategy
//!
//! * **Nodes** are matched by their `name` field (must be unique within a
//!   graph; the caller is responsible for disambiguation).
//! * **Edges** are matched by the triple `(source_name, target_name,
//!   edge_kind)`.
//!
//! # Example
//!
//! ```rust,no_run
//! use icb_server::diff::{diff_graphs, Status};
//! use icb_graph::graph::CodePropertyGraph;
//!
//! let old = CodePropertyGraph::new();
//! let new = CodePropertyGraph::new();
//! let report = diff_graphs(&old, &new);
//! assert!(report.nodes.is_empty());
//! ```

use icb_graph::graph::CodePropertyGraph;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde::Serialize;
use std::collections::HashSet;

/// Status of an entity in the diff.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Status {
    /// Present only in the new graph.
    Added,
    /// Present only in the old graph.
    Removed,
    /// Present in both graphs.
    Unchanged,
}

/// A single node in the diff output.
#[derive(Debug, Serialize)]
pub struct DiffNode {
    /// Display name of the node.
    pub name: String,
    /// Node kind (e.g. `Function`, `Class`).
    pub kind: String,
    /// Starting line in the source file.
    pub line: usize,
    /// Associated file path (or USR).
    pub file: String,
    /// Diff status.
    pub status: Status,
}

/// A single edge in the diff output.
#[derive(Debug, Serialize)]
pub struct DiffEdge {
    /// Source node name.
    pub source: String,
    /// Target node name.
    pub target: String,
    /// Edge kind (e.g. `Call`, `AstChild`).
    pub kind: String,
    /// Diff status.
    pub status: Status,
}

/// Complete result of a diff operation.
#[derive(Debug, Serialize)]
pub struct DiffReport {
    /// All nodes from both graphs, each tagged with its status.
    pub nodes: Vec<DiffNode>,
    /// All unique edges from both graphs, tagged with status.
    pub edges: Vec<DiffEdge>,
}

/// Compute the symmetric difference between two graphs.
///
/// Nodes are matched by their `name` field.  Edges are matched by the
/// ordered triple `(source_name, target_name, edge_kind)`.
pub fn diff_graphs(old: &CodePropertyGraph, new: &CodePropertyGraph) -> DiffReport {
    let mut report = DiffReport {
        nodes: Vec::new(),
        edges: Vec::new(),
    };

    let old_names: HashSet<String> = old
        .graph
        .node_weights()
        .map(|n| n.name.clone().unwrap_or_default())
        .collect();
    let new_names: HashSet<String> = new
        .graph
        .node_weights()
        .map(|n| n.name.clone().unwrap_or_default())
        .collect();

    let all_names: HashSet<&str> = old_names
        .iter()
        .chain(new_names.iter())
        .map(|s| s.as_str())
        .collect();

    for name in all_names {
        let in_old = old_names.contains(name);
        let in_new = new_names.contains(name);
        let (status, node_ref) = if in_old && in_new {
            let node = old
                .graph
                .node_weights()
                .find(|n| n.name.as_deref() == Some(name))
                .unwrap();
            (Status::Unchanged, node)
        } else if in_old {
            let node = old
                .graph
                .node_weights()
                .find(|n| n.name.as_deref() == Some(name))
                .unwrap();
            (Status::Removed, node)
        } else {
            let node = new
                .graph
                .node_weights()
                .find(|n| n.name.as_deref() == Some(name))
                .unwrap();
            (Status::Added, node)
        };

        report.nodes.push(DiffNode {
            name: name.to_string(),
            kind: format!("{:?}", node_ref.kind),
            line: node_ref.start_line,
            file: node_ref.usr.clone().unwrap_or_default(),
            status,
        });
    }

    let mut old_edge_triples: HashSet<(String, String, String)> = HashSet::new();
    let mut new_edge_triples: HashSet<(String, String, String)> = HashSet::new();

    for edge_ref in old.graph.edge_references() {
        let src = old.graph[edge_ref.source()]
            .name
            .clone()
            .unwrap_or_default();
        let tgt = old.graph[edge_ref.target()]
            .name
            .clone()
            .unwrap_or_default();
        let kind = format!("{:?}", edge_ref.weight());
        old_edge_triples.insert((src, tgt, kind));
    }
    for edge_ref in new.graph.edge_references() {
        let src = new.graph[edge_ref.source()]
            .name
            .clone()
            .unwrap_or_default();
        let tgt = new.graph[edge_ref.target()]
            .name
            .clone()
            .unwrap_or_default();
        let kind = format!("{:?}", edge_ref.weight());
        new_edge_triples.insert((src, tgt, kind));
    }

    for (src, tgt, kind) in new_edge_triples.difference(&old_edge_triples) {
        report.edges.push(DiffEdge {
            source: src.clone(),
            target: tgt.clone(),
            kind: kind.clone(),
            status: Status::Added,
        });
    }
    for (src, tgt, kind) in old_edge_triples.difference(&new_edge_triples) {
        report.edges.push(DiffEdge {
            source: src.clone(),
            target: tgt.clone(),
            kind: kind.clone(),
            status: Status::Removed,
        });
    }

    report
}
