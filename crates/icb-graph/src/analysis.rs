use crate::graph::{CodePropertyGraph, Edge, Node};
use icb_common::NodeKind;
use petgraph::algo::kosaraju_scc;
use petgraph::graph::Graph;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use std::collections::{HashMap, HashSet, VecDeque};

/// A cycle of function calls.
#[derive(Debug, Clone)]
pub struct CallCycle {
    /// Function names involved in the cycle, in arbitrary order.
    pub functions: Vec<String>,
    /// Number of distinct functions in the cycle.
    pub length: usize,
}

/// Detects all call cycles involving one or more functions.
///
/// Builds a directed graph of function nodes connected by [`Edge::Call`] edges
/// and runs Kosaraju's algorithm for strongly connected components (SCC).
/// Every SCC with more than one node is reported as a cycle. Self‑loops
/// (single‑node SCCs with a call to themselves) are also reported.
///
/// # Examples
///
/// ```rust
/// use icb_graph::analysis::detect_call_cycles;
/// use icb_graph::graph::{CodePropertyGraph, Node, Edge};
/// use icb_common::NodeKind;
///
/// let mut cpg = CodePropertyGraph::new();
/// let a = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("a".into()),
///     usr: Some("a".into()),
///     start_line: 1, end_line: 1,
/// });
/// let b = cpg.graph.add_node(Node {
///     kind: NodeKind::Function,
///     name: Some("b".into()),
///     usr: Some("b".into()),
///     start_line: 2, end_line: 2,
/// });
/// cpg.graph.add_edge(a, b, Edge::Call);
/// cpg.graph.add_edge(b, a, Edge::Call);
///
/// let cycles = detect_call_cycles(&cpg);
/// assert_eq!(cycles.len(), 1);
/// assert!(cycles[0].functions.contains(&"a".into()));
/// ```
pub fn detect_call_cycles(cpg: &CodePropertyGraph) -> Vec<CallCycle> {
    let mut call_graph = Graph::<String, (), petgraph::Directed>::new();
    let mut node_map: HashMap<petgraph::stable_graph::NodeIndex, petgraph::graph::NodeIndex> =
        HashMap::new();

    for node_idx in cpg.graph.node_indices() {
        let node = &cpg.graph[node_idx];
        if node.kind == NodeKind::Function || node.kind == NodeKind::Class {
            let name = node.name.clone().unwrap_or_else(|| "?".into());
            let gidx = call_graph.add_node(name);
            node_map.insert(node_idx, gidx);
        }
    }

    for edge_ref in cpg.graph.edge_references() {
        if *edge_ref.weight() != Edge::Call {
            continue;
        }
        let source = edge_ref.source();
        let target = edge_ref.target();
        if let (Some(&src_g), Some(&tgt_g)) = (node_map.get(&source), node_map.get(&target)) {
            call_graph.add_edge(src_g, tgt_g, ());
        }
    }

    let sccs = kosaraju_scc(&call_graph);
    let mut cycles = Vec::new();

    for scc in sccs {
        if scc.len() > 1 {
            let names: Vec<String> = scc.iter().map(|&i| call_graph[i].clone()).collect();
            cycles.push(CallCycle {
                length: names.len(),
                functions: names,
            });
        } else if scc.len() == 1 {
            let gidx = scc[0];
            // check self-loop
            if call_graph.contains_edge(gidx, gidx) {
                cycles.push(CallCycle {
                    functions: vec![call_graph[gidx].clone()],
                    length: 1,
                });
            }
        }
    }

    cycles
}

/// A function complexity report.
#[derive(Debug, Clone)]
pub struct ComplexityReport {
    /// Name of the function.
    pub function_name: String,
    /// Total number of AST nodes contained in the function's body.
    pub ast_node_count: usize,
    /// Start line of the function.
    pub start_line: usize,
}

/// Detects functions whose AST subtree contains more than `threshold` nodes.
///
/// The function collects all function nodes, traverses the AST subtree of each
/// (following [`Edge::AstChild`] edges) and counts the number of descendant
/// nodes. Functions exceeding the threshold are returned.
///
/// # Arguments
///
/// * `cpg` - The Code Property Graph.
/// * `threshold` - Maximum allowed AST nodes before a function is considered complex.
pub fn detect_complex_functions(
    cpg: &CodePropertyGraph,
    threshold: usize,
) -> Vec<ComplexityReport> {
    let mut results = Vec::new();
    for node_idx in cpg.graph.node_indices() {
        let node = &cpg.graph[node_idx];
        if node.kind != NodeKind::Function {
            continue;
        }
        let count = count_subtree_nodes(cpg, node_idx);
        if count > threshold {
            results.push(ComplexityReport {
                function_name: node.name.clone().unwrap_or_else(|| "?".into()),
                ast_node_count: count,
                start_line: node.start_line,
            });
        }
    }
    results
}

fn count_subtree_nodes(cpg: &CodePropertyGraph, root: petgraph::stable_graph::NodeIndex) -> usize {
    let mut count = 0;
    let mut queue = VecDeque::new();
    queue.push_back(root);
    while let Some(current) = queue.pop_front() {
        count += 1;
        for edge_ref in cpg.graph.edges(current) {
            if *edge_ref.weight() == Edge::AstChild {
                queue.push_back(edge_ref.target());
            }
        }
    }
    count
}

/// Finds functions that are not reachable from any of the specified entry
/// functions via call edges.
///
/// # Arguments
///
/// * `cpg` - The Code Property Graph.
/// * `entry_names` - Names of functions that serve as entry points (e.g. `["main"]`).
///
/// # Returns
///
/// A vector of references to unreachable function nodes. If no entry function
/// is found, all functions are considered unreachable.
pub fn detect_dead_code<'a>(cpg: &'a CodePropertyGraph, entry_names: &[String]) -> Vec<&'a Node> {
    // Build a mapping from name to node indices (there could be duplicates)
    let mut name_to_idx: HashMap<String, Vec<petgraph::stable_graph::NodeIndex>> = HashMap::new();
    for idx in cpg.graph.node_indices() {
        let node = &cpg.graph[idx];
        if node.kind == NodeKind::Function || node.kind == NodeKind::Class {
            if let Some(name) = &node.name {
                name_to_idx.entry(name.clone()).or_default().push(idx);
            }
        }
    }

    let entry_indices: Vec<petgraph::stable_graph::NodeIndex> = entry_names
        .iter()
        .filter_map(|name| name_to_idx.get(name).and_then(|v| v.first().copied()))
        .collect();

    if entry_indices.is_empty() {
        return cpg
            .graph
            .node_weights()
            .filter(|n| n.kind == NodeKind::Function)
            .collect();
    }

    let mut reachable = HashSet::new();
    let mut queue = VecDeque::from(entry_indices);
    while let Some(current) = queue.pop_front() {
        if !reachable.insert(current) {
            continue;
        }
        for edge_ref in cpg.graph.edges(current) {
            if *edge_ref.weight() == Edge::Call {
                queue.push_back(edge_ref.target());
            }
        }
    }

    cpg.graph
        .node_indices()
        .filter_map(|idx| {
            let node = &cpg.graph[idx];
            if node.kind == NodeKind::Function && !reachable.contains(&idx) {
                Some(node)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;
    use icb_common::NodeKind;

    fn build_test_cpg() -> CodePropertyGraph {
        let mut cpg = CodePropertyGraph::new();
        let a = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("a".into()),
            usr: Some("a".into()),
            start_line: 1,
            end_line: 1,
        });
        let b = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("b".into()),
            usr: Some("b".into()),
            start_line: 2,
            end_line: 2,
        });
        let c = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("c".into()),
            usr: Some("c".into()),
            start_line: 3,
            end_line: 3,
        });
        cpg.graph.add_edge(a, b, Edge::Call);
        cpg.graph.add_edge(b, c, Edge::Call);
        cpg.graph.add_edge(c, a, Edge::Call);

        let d = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("d".into()),
            usr: Some("d".into()),
            start_line: 4,
            end_line: 4,
        });
        cpg.graph.add_edge(d, d, Edge::Call);

        cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("e".into()),
            usr: Some("e".into()),
            start_line: 5,
            end_line: 5,
        });

        let f = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("f".into()),
            usr: Some("f".into()),
            start_line: 6,
            end_line: 6,
        });
        let mut last = f;
        for _ in 0..10 {
            let child = cpg.graph.add_node(Node {
                kind: NodeKind::Variable,
                name: None,
                usr: None,
                start_line: 6,
                end_line: 6,
            });
            cpg.graph.add_edge(last, child, Edge::AstChild);
            last = child;
        }

        cpg
    }

    #[test]
    fn test_detect_call_cycles() {
        let cpg = build_test_cpg();
        let cycles = detect_call_cycles(&cpg);
        assert_eq!(cycles.len(), 2);
        let three_cycle = cycles.iter().find(|c| c.length == 3).unwrap();
        assert!(three_cycle.functions.contains(&"a".into()));
        assert!(three_cycle.functions.contains(&"b".into()));
        assert!(three_cycle.functions.contains(&"c".into()));
        let self_loop = cycles.iter().find(|c| c.length == 1).unwrap();
        assert_eq!(self_loop.functions, vec!["d"]);
    }

    #[test]
    fn test_dead_code_from_entry() {
        let cpg = build_test_cpg();
        let dead = detect_dead_code(&cpg, &["a".to_string()]);
        // d, e, f are unreachable from a
        assert!(dead.iter().any(|n| n.name.as_deref() == Some("d")));
        assert!(dead.iter().any(|n| n.name.as_deref() == Some("e")));
        assert!(dead.iter().any(|n| n.name.as_deref() == Some("f")));
        // a, b, c are reachable
        assert!(!dead.iter().any(|n| n.name.as_deref() == Some("a")));
        assert!(!dead.iter().any(|n| n.name.as_deref() == Some("b")));
        assert!(!dead.iter().any(|n| n.name.as_deref() == Some("c")));
    }

    #[test]
    fn test_complex_functions() {
        let cpg = build_test_cpg();
        let complex = detect_complex_functions(&cpg, 5);
        assert_eq!(complex.len(), 1);
        assert_eq!(complex[0].function_name, "f");
        assert!(complex[0].ast_node_count > 10);
    }
}
