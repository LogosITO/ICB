use icb_common::NodeKind;
use icb_graph::analysis;
use icb_graph::graph::{CodePropertyGraph, Edge};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct FunctionMetric {
    pub name: String,
    pub kind: String,
    pub line: usize,
    pub file: Option<String>,
    pub complexity: usize,
    pub is_cycle: bool,
    pub is_dead: bool,
    pub callers: usize,
    pub callees: usize,
}

pub fn collect_function_metrics(cpg: &CodePropertyGraph) -> Vec<FunctionMetric> {
    let cycles = analysis::detect_call_cycles(cpg);
    let dead = analysis::detect_dead_code(cpg, &["main".to_string()]);
    let complex_list = analysis::detect_complex_functions(cpg, 0);

    cpg.graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::Function || n.kind == NodeKind::Class)
        .map(|node| {
            let raw_name = node.name.clone().unwrap_or_default();

            // Clean USR-like names: take the last segment after '@' and remove '#...' suffix
            let name = if raw_name.contains('@') && raw_name.contains('#') {
                raw_name
                    .split('@')
                    .next_back()
                    .unwrap_or(&raw_name)
                    .split('#')
                    .next()
                    .unwrap_or(&raw_name)
                    .to_string()
            } else {
                raw_name
            };

            let is_cycle = cycles.iter().any(|c| c.functions.contains(&name));
            let is_dead = dead.iter().any(|n| n.name.as_deref() == Some(&name));
            let complexity = complex_list
                .iter()
                .find(|r| r.function_name == name)
                .map(|r| r.ast_node_count)
                .unwrap_or(0);

            let idx = cpg
                .graph
                .node_indices()
                .find(|&i| cpg.graph[i].name == node.name)
                .unwrap();

            let callers = cpg
                .graph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .filter(|e| matches!(e.weight(), Edge::Call))
                .count();
            let callees = cpg
                .graph
                .edges(idx)
                .filter(|e| matches!(e.weight(), Edge::Call))
                .count();

            FunctionMetric {
                name,
                kind: format!("{:?}", node.kind),
                line: node.start_line,
                file: None,
                complexity,
                is_cycle,
                is_dead,
                callers,
                callees,
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct ClassMetric {
    pub name: String,
    pub line: usize,
    pub file: Option<String>,
    pub methods: usize,
    pub complexity: usize,
}

pub fn collect_class_metrics(cpg: &CodePropertyGraph) -> Vec<ClassMetric> {
    let complex_list = analysis::detect_complex_functions(cpg, 0);

    cpg.graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::Class)
        .map(|node| {
            let name = node.name.clone().unwrap_or_default();
            let complexity = complex_list
                .iter()
                .find(|r| r.function_name == name)
                .map(|r| r.ast_node_count)
                .unwrap_or(0);

            let idx = cpg
                .graph
                .node_indices()
                .find(|&i| cpg.graph[i].name == node.name)
                .unwrap();

            let methods = cpg
                .graph
                .edges(idx)
                .filter(|e| matches!(e.weight(), Edge::AstChild))
                .filter(|e| cpg.graph[e.target()].kind == NodeKind::Function)
                .count();

            ClassMetric {
                name,
                line: node.start_line,
                file: None,
                methods,
                complexity,
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct FileMetric {
    pub path: String,
    pub functions: usize,
    pub classes: usize,
    pub total_complexity: usize,
    pub calls: usize,
}

pub fn collect_file_metrics(cpg: &CodePropertyGraph) -> Vec<FileMetric> {
    let mut files: HashMap<String, (usize, usize, usize, usize)> = HashMap::new();
    for node in cpg.graph.node_weights() {
        let file = node.usr.clone().unwrap_or_default();
        let entry = files.entry(file).or_insert((0, 0, 0, 0));
        match node.kind {
            NodeKind::Function => entry.0 += 1,
            NodeKind::Class => entry.1 += 1,
            _ => {}
        }
    }
    for edge_ref in cpg.graph.edge_references() {
        if matches!(edge_ref.weight(), Edge::Call) {
            let src_node = &cpg.graph[edge_ref.source()];
            let file = src_node.usr.clone().unwrap_or_default();
            let entry = files.entry(file).or_insert((0, 0, 0, 0));
            entry.3 += 1;
        }
    }
    files
        .into_iter()
        .map(|(path, (funcs, classes, _compl, calls))| FileMetric {
            path,
            functions: funcs,
            classes,
            total_complexity: 0,
            calls,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use icb_graph::graph::{Edge, Node};

    fn build_test_cpg() -> CodePropertyGraph {
        let mut cpg = CodePropertyGraph::new();
        let main = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("main".into()),
            usr: Some("main.cpp".into()),
            start_line: 1,
            end_line: 5,
        });
        let helper = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("helper".into()),
            usr: Some("helper.cpp".into()),
            start_line: 10,
            end_line: 12,
        });
        let my_class = cpg.graph.add_node(Node {
            kind: NodeKind::Class,
            name: Some("MyClass".into()),
            usr: Some("myclass.cpp".into()),
            start_line: 20,
            end_line: 30,
        });
        let method = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("method".into()),
            usr: Some("myclass.cpp".into()),
            start_line: 22,
            end_line: 25,
        });

        cpg.graph.add_edge(main, helper, Edge::Call);
        cpg.graph.add_edge(my_class, method, Edge::AstChild);
        cpg
    }

    #[test]
    fn test_function_metrics() {
        let cpg = build_test_cpg();
        let metrics = collect_function_metrics(&cpg);
        assert_eq!(metrics.len(), 4);
        let main_metric = metrics.iter().find(|m| m.name == "main").unwrap();
        assert_eq!(main_metric.callees, 1);
        assert_eq!(main_metric.callers, 0);
        let helper_metric = metrics.iter().find(|m| m.name == "helper").unwrap();
        assert_eq!(helper_metric.callers, 1);
    }

    #[test]
    fn test_class_metrics() {
        let cpg = build_test_cpg();
        let metrics = collect_class_metrics(&cpg);
        assert_eq!(metrics.len(), 1);
        let class_metric = &metrics[0];
        assert_eq!(class_metric.name, "MyClass");
        assert_eq!(class_metric.methods, 1);
    }

    #[test]
    fn test_file_metrics() {
        let cpg = build_test_cpg();
        let metrics = collect_file_metrics(&cpg);
        assert_eq!(metrics.len(), 3);
        let main_file = metrics.iter().find(|f| f.path == "main.cpp").unwrap();
        assert_eq!(main_file.functions, 1);
        assert_eq!(main_file.calls, 1);
    }
}
