use crate::graph::{CodePropertyGraph, Edge, Node};
use icb_common::NodeKind;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::{HashSet, VecDeque};

/// Return all nodes of the given `kind`.
pub fn find_by_kind(cpg: &CodePropertyGraph, kind: NodeKind) -> Vec<&Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == kind)
        .collect()
}

/// Return all call sites that target a function with the given `func_name`.
pub fn find_calls_to<'a>(cpg: &'a CodePropertyGraph, func_name: &str) -> Vec<&'a Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::CallSite && n.name.as_deref() == Some(func_name))
        .collect()
}

/// For a given function name, return all functions that directly call it.
/// Each result is `(caller_function_node, callee_def_node)`.
pub fn callers_of<'a>(cpg: &'a CodePropertyGraph, func_name: &str) -> Vec<(&'a Node, &'a Node)> {
    let target_defs: HashSet<_> = cpg
        .graph
        .node_indices()
        .filter(|&idx| {
            let node = &cpg.graph[idx];
            (node.kind == NodeKind::Function || node.kind == NodeKind::Class)
                && node.name.as_deref() == Some(func_name)
        })
        .collect();

    if target_defs.is_empty() {
        return vec![];
    }

    let mut results = Vec::new();
    for edge_ref in cpg.graph.edge_references() {
        if *edge_ref.weight() != Edge::Call {
            continue;
        }
        let call_idx = edge_ref.source();
        let def_idx = edge_ref.target();
        if target_defs.contains(&def_idx) {
            // Find enclosing function of the call site
            if let Some(enclosing) = get_enclosing_function(cpg, call_idx) {
                results.push((enclosing, &cpg.graph[def_idx]));
            }
        }
    }
    results
}

/// For a given function name, return all functions it directly calls.
/// Each result is `(callee_node, call_site_node)`.
pub fn callees_of<'a>(cpg: &'a CodePropertyGraph, func_name: &str) -> Vec<(&'a Node, &'a Node)> {
    let caller_idx = cpg.graph.node_indices().find(|&idx| {
        let node = &cpg.graph[idx];
        node.kind == NodeKind::Function && node.name.as_deref() == Some(func_name)
    });

    let caller_idx = match caller_idx {
        Some(idx) => idx,
        None => return vec![],
    };

    let mut results = Vec::new();
    // Collect all call sites inside the function's AST subtree
    let call_sites = collect_call_sites_in_subtree(cpg, caller_idx);

    for call_idx in call_sites {
        // For each call site, find outgoing Call edges
        for edge_ref in cpg.graph.edges(call_idx) {
            if *edge_ref.weight() == Edge::Call {
                results.push((&cpg.graph[edge_ref.target()], &cpg.graph[call_idx]));
            }
        }
    }
    results
}

/// Return all functions that are never called directly.
pub fn unused_functions(cpg: &CodePropertyGraph) -> Vec<&Node> {
    let called_defs: HashSet<_> = cpg
        .graph
        .edge_references()
        .filter(|e| *e.weight() == Edge::Call)
        .map(|e| e.target())
        .collect();

    cpg.graph
        .node_indices()
        .filter_map(|idx| {
            let node = &cpg.graph[idx];
            if node.kind == NodeKind::Function && !called_defs.contains(&idx) {
                Some(node)
            } else {
                None
            }
        })
        .collect()
}

// ── Helpers ──────────────────────────────────────────────────────

/// Walk up AST edges from a node until we hit a function definition.
fn get_enclosing_function(
    cpg: &CodePropertyGraph,
    start: petgraph::stable_graph::NodeIndex,
) -> Option<&Node> {
    let mut current = start;
    loop {
        let node = &cpg.graph[current];
        if node.kind == NodeKind::Function {
            return Some(node);
        }
        // find parent via AstChild edge (incoming)
        let parent = cpg
            .graph
            .edges_directed(current, petgraph::Direction::Incoming)
            .find(|e| *e.weight() == Edge::AstChild)
            .map(|e| e.source());
        match parent {
            Some(p) => current = p,
            None => return None,
        }
    }
}

/// Collect all node indices of type CallSite that are in the AST subtree of `root`.
fn collect_call_sites_in_subtree(
    cpg: &CodePropertyGraph,
    root: petgraph::stable_graph::NodeIndex,
) -> Vec<petgraph::stable_graph::NodeIndex> {
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(root);
    while let Some(current) = queue.pop_front() {
        if cpg.graph[current].kind == NodeKind::CallSite {
            result.push(current);
        }
        for edge_ref in cpg.graph.edges(current) {
            if *edge_ref.weight() == Edge::AstChild {
                queue.push_back(edge_ref.target());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Edge;
    use icb_common::NodeKind;

    fn make_test_graph() -> CodePropertyGraph {
        let mut cpg = CodePropertyGraph::new();
        // Create functions: foo (line 1) and bar (line 10)
        let foo = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("foo".into()),
            usr: Some("foo".into()),
            start_line: 1,
            end_line: 3,
        });
        let bar = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("bar".into()),
            usr: Some("bar".into()),
            start_line: 10,
            end_line: 12,
        });

        // Call site inside foo that calls bar
        let call_in_foo = cpg.graph.add_node(Node {
            kind: NodeKind::CallSite,
            name: Some("bar".into()),
            usr: None,
            start_line: 2,
            end_line: 2,
        });
        cpg.graph.add_edge(foo, call_in_foo, Edge::AstChild);
        cpg.graph.add_edge(call_in_foo, bar, Edge::Call);

        // Call site inside bar that calls foo
        let call_in_bar = cpg.graph.add_node(Node {
            kind: NodeKind::CallSite,
            name: Some("foo".into()),
            usr: None,
            start_line: 11,
            end_line: 11,
        });
        cpg.graph.add_edge(bar, call_in_bar, Edge::AstChild);
        cpg.graph.add_edge(call_in_bar, foo, Edge::Call);

        // Another function: baz (line 20), unused
        cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("baz".into()),
            usr: Some("baz".into()),
            start_line: 20,
            end_line: 22,
        });

        cpg
    }

    #[test]
    fn test_callers_of() {
        let cpg = make_test_graph();
        // Callers of "bar" should be "foo"
        let callers_bar = callers_of(&cpg, "bar");
        assert_eq!(callers_bar.len(), 1);
        assert_eq!(callers_bar[0].0.name.as_deref(), Some("foo"));
        // Callers of "foo" should be "bar"
        let callers_foo = callers_of(&cpg, "foo");
        assert_eq!(callers_foo.len(), 1);
        assert_eq!(callers_foo[0].0.name.as_deref(), Some("bar"));
    }

    #[test]
    fn test_callees_of() {
        let cpg = make_test_graph();
        // Callees of "foo" should be "bar"
        let callees_foo = callees_of(&cpg, "foo");
        assert_eq!(callees_foo.len(), 1);
        assert_eq!(callees_foo[0].0.name.as_deref(), Some("bar"));
        // Callees of "bar" should be "foo"
        let callees_bar = callees_of(&cpg, "bar");
        assert_eq!(callees_bar.len(), 1);
        assert_eq!(callees_bar[0].0.name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_unused_functions() {
        let cpg = make_test_graph();
        let unused = unused_functions(&cpg);
        // Only "baz" should be unused
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].name.as_deref(), Some("baz"));
    }
}
