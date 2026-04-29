use crate::graph::{CodePropertyGraph, Edge, Node};
use icb_common::NodeKind;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::{HashSet, VecDeque};

/// Returns all nodes of the given `kind`.
///
/// Traverses the entire graph and collects every node whose
/// [`NodeKind`] matches the requested kind.
///
/// # Examples
///
/// ```rust
/// use icb_graph::graph::{CodePropertyGraph, Node};
/// use icb_graph::query::find_by_kind;
/// use icb_common::NodeKind;
///
/// let mut cpg = CodePropertyGraph::new();
/// cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("main".into()),
///     usr: None,
///     start_line: 1,
///     end_line: 5,
/// });
///
/// let functions = find_by_kind(&cpg, NodeKind::Function);
/// assert_eq!(functions.len(), 1);
/// ```
pub fn find_by_kind(cpg: &CodePropertyGraph, kind: NodeKind) -> Vec<&Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == kind)
        .collect()
}

/// Returns all call sites that target a function with the given `func_name`.
///
/// The comparison is done only on the node's name; no overload resolution
/// is performed.
///
/// # Examples
///
/// ```rust
/// use icb_graph::graph::{CodePropertyGraph, Node};
/// use icb_graph::query::find_calls_to;
/// use icb_common::NodeKind;
///
/// let mut cpg = CodePropertyGraph::new();
/// cpg.graph.add_node(Node {
///     kind: NodeKind::CallSite,
///     name: Some("foo".into()),
///     usr: None,
///     start_line: 1,
///     end_line: 1,
/// });
///
/// let calls = find_calls_to(&cpg, "foo");
/// assert_eq!(calls.len(), 1);
/// ```
pub fn find_calls_to<'a>(cpg: &'a CodePropertyGraph, func_name: &str) -> Vec<&'a Node> {
    cpg.graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::CallSite && n.name.as_deref() == Some(func_name))
        .collect()
}

/// For a given function name, returns all functions that directly call it.
///
/// Each returned tuple contains the caller function node and the callee
/// definition node. The enclosing function is found by walking up
/// [`Edge::AstChild`] edges from the call site.
///
/// # Examples
///
/// ```rust
/// use icb_graph::graph::{CodePropertyGraph, Node, Edge};
/// use icb_graph::query::callers_of;
/// use icb_common::NodeKind;
///
/// let mut cpg = CodePropertyGraph::new();
/// let foo = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("foo".into()),
///     usr: Some("foo".into()),
///     start_line: 1,
///     end_line: 2,
/// });
/// let bar = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("bar".into()),
///     usr: Some("bar".into()),
///     start_line: 5,
///     end_line: 6,
/// });
/// let call = cpg.graph.add_node(Node {
///     kind: NodeKind::CallSite,
///     name: Some("bar".into()),
///     usr: None,
///     start_line: 2,
///     end_line: 2,
/// });
/// cpg.graph.add_edge(foo, call, Edge::AstChild);
/// cpg.graph.add_edge(call, bar, Edge::Call);
///
/// let callers = callers_of(&cpg, "bar");
/// assert_eq!(callers.len(), 1);
/// assert_eq!(callers[0].0.name.as_deref(), Some("foo"));
/// ```
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
            if let Some(enclosing) = get_enclosing_function(cpg, call_idx) {
                results.push((enclosing, &cpg.graph[def_idx]));
            }
        }
    }
    results
}

/// For a given function name, returns all functions it directly calls.
///
/// Each returned tuple contains the callee node and the call‑site node.
/// The function first locates the caller by name, collects all
/// [`NodeKind::CallSite`] nodes inside its AST subtree, and follows
/// outgoing [`Edge::Call`] edges.
///
/// # Examples
///
/// ```rust
/// use icb_graph::graph::{CodePropertyGraph, Node, Edge};
/// use icb_graph::query::callees_of;
/// use icb_common::NodeKind;
///
/// let mut cpg = CodePropertyGraph::new();
/// let foo = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("foo".into()),
///     usr: Some("foo".into()),
///     start_line: 1,
///     end_line: 2,
/// });
/// let bar = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("bar".into()),
///     usr: Some("bar".into()),
///     start_line: 5,
///     end_line: 6,
/// });
/// let call = cpg.graph.add_node(Node {
///     kind: NodeKind::CallSite,
///     name: Some("bar".into()),
///     usr: None,
///     start_line: 2,
///     end_line: 2,
/// });
/// cpg.graph.add_edge(foo, call, Edge::AstChild);
/// cpg.graph.add_edge(call, bar, Edge::Call);
///
/// let callees = callees_of(&cpg, "foo");
/// assert_eq!(callees.len(), 1);
/// assert_eq!(callees[0].0.name.as_deref(), Some("bar"));
/// ```
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
    let call_sites = collect_call_sites_in_subtree(cpg, caller_idx);

    for call_idx in call_sites {
        for edge_ref in cpg.graph.edges(call_idx) {
            if *edge_ref.weight() == Edge::Call {
                results.push((&cpg.graph[edge_ref.target()], &cpg.graph[call_idx]));
            }
        }
    }
    results
}

/// Returns all functions that are never the target of a direct call.
///
/// A function is considered unused if no [`Edge::Call`] edge points to its
/// node. This is a simple direct‑call analysis; indirect calls through
/// function pointers or references are not detected.
///
/// # Examples
///
/// ```rust
/// use icb_graph::graph::{CodePropertyGraph, Node, Edge};
/// use icb_graph::query::unused_functions;
/// use icb_common::NodeKind;
///
/// let mut cpg = CodePropertyGraph::new();
/// let foo = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("foo".into()),
///     usr: Some("foo".into()),
///     start_line: 1,
///     end_line: 1,
/// });
/// let bar = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("bar".into()),
///     usr: Some("bar".into()),
///     start_line: 2,
///     end_line: 2,
/// });
/// // foo calls bar
/// cpg.graph.add_edge(foo, bar, Edge::Call);
///
/// let unused = unused_functions(&cpg);
/// // foo is never called directly => unused
/// assert!(unused.iter().any(|n| n.name.as_deref() == Some("foo")));
/// assert!(!unused.iter().any(|n| n.name.as_deref() == Some("bar")));
/// ```
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

/// Walks up [`Edge::AstChild`] edges from `start` until a
/// [`NodeKind::Function`] node is found.
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

/// Collects all [`NodeKind::CallSite`] nodes in the AST subtree of `root`.
///
/// The traversal follows [`Edge::AstChild`] edges.
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

        let call_in_foo = cpg.graph.add_node(Node {
            kind: NodeKind::CallSite,
            name: Some("bar".into()),
            usr: None,
            start_line: 2,
            end_line: 2,
        });
        cpg.graph.add_edge(foo, call_in_foo, Edge::AstChild);
        cpg.graph.add_edge(call_in_foo, bar, Edge::Call);

        let call_in_bar = cpg.graph.add_node(Node {
            kind: NodeKind::CallSite,
            name: Some("foo".into()),
            usr: None,
            start_line: 11,
            end_line: 11,
        });
        cpg.graph.add_edge(bar, call_in_bar, Edge::AstChild);
        cpg.graph.add_edge(call_in_bar, foo, Edge::Call);

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
        let callers_bar = callers_of(&cpg, "bar");
        assert_eq!(callers_bar.len(), 1);
        assert_eq!(callers_bar[0].0.name.as_deref(), Some("foo"));

        let callers_foo = callers_of(&cpg, "foo");
        assert_eq!(callers_foo.len(), 1);
        assert_eq!(callers_foo[0].0.name.as_deref(), Some("bar"));
    }

    #[test]
    fn test_callees_of() {
        let cpg = make_test_graph();
        let callees_foo = callees_of(&cpg, "foo");
        assert_eq!(callees_foo.len(), 1);
        assert_eq!(callees_foo[0].0.name.as_deref(), Some("bar"));

        let callees_bar = callees_of(&cpg, "bar");
        assert_eq!(callees_bar.len(), 1);
        assert_eq!(callees_bar[0].0.name.as_deref(), Some("foo"));
    }

    #[test]
    fn test_unused_functions() {
        let cpg = make_test_graph();
        let unused = unused_functions(&cpg);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].name.as_deref(), Some("baz"));
    }
}
