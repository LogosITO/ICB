#![allow(dead_code)]

//! Generators of synthetic facts for graph benchmarks.
//!
//! Each function returns a `Vec<RawNode>` that simulates a C++ file
//! with the specified number of functions and call relationships.

use icb_common::{Language, NodeKind};
use icb_parser::facts::RawNode;

/// Create facts for `n` functions, each containing a single call to a random other function.
/// Returns a flat list of facts that can be fed to `GraphBuilder::ingest_file_facts`.
pub fn generate_facts(num_functions: usize) -> Vec<RawNode> {
    let mut nodes = Vec::with_capacity(num_functions * 2); // function + call site
    let mut rng = fastrand::Rng::new();

    // First pass: create all function nodes
    for i in 0..num_functions {
        nodes.push(RawNode {
            language: Language::Cpp,
            kind: NodeKind::Function,
            name: Some(format!("func{}", i)),
            usr: Some(format!("c:@F@func{}#", i)),
            start_line: (i * 2 + 1) as usize,
            start_col: 0,
            end_line: (i * 2 + 1) as usize,
            end_col: 10,
            children: vec![],
            source_file: Some("bench.cpp".into()),
        });
    }

    // Second pass: add call sites inside each function
    for i in 0..num_functions {
        let callee_idx = rng.usize(0..num_functions);
        nodes.push(RawNode {
            language: Language::Cpp,
            kind: NodeKind::CallSite,
            name: Some(format!("func{}", callee_idx)),
            usr: None,
            start_line: (i * 2 + 2) as usize,
            start_col: 4,
            end_line: (i * 2 + 2) as usize,
            end_col: 20,
            children: vec![],
            source_file: Some("bench.cpp".into()),
        });
    }

    nodes
}

/// Create a `GraphBuilder` pre‑filled with `n` functions and calls.
pub fn build_graph(n: usize) -> icb_graph::graph::CodePropertyGraph {
    let facts = generate_facts(n);
    let mut builder = icb_graph::builder::GraphBuilder::new();
    builder.ingest_file_facts(&facts);
    builder.resolve_calls();
    builder.cpg
}
