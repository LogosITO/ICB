use criterion::{criterion_group, criterion_main, Criterion};
use icb_common::NodeKind;
use icb_graph::graph::{CodePropertyGraph, Edge, GraphData};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use std::hint::black_box;

fn make_test_graph(size: usize) -> CodePropertyGraph {
    let mut cpg = CodePropertyGraph::new();
    let mut indices = vec![];
    for i in 0..size {
        indices.push(cpg.graph.add_node(icb_graph::graph::Node {
            kind: NodeKind::Function,
            name: Some(format!("func{}", i)),
            usr: Some("bench.cpp".into()),
            start_line: i + 1,
            end_line: i + 1,
        }));
    }
    for i in 1..size {
        cpg.graph.add_edge(indices[i - 1], indices[i], Edge::Call);
    }
    cpg
}

fn focal_graph_for_bench(
    cpg: &CodePropertyGraph,
    func_name: &str,
    max_nodes: usize,
    depth: usize,
) -> GraphData {
    let mut included = HashSet::new();
    let mut frontier = Vec::new();
    for idx in cpg.graph.node_indices() {
        let node = &cpg.graph[idx];
        if node.kind == NodeKind::Function && node.name.as_deref() == Some(func_name) {
            included.insert(idx.index());
            frontier.push(idx);
        }
    }
    for _ in 0..depth {
        let mut next = Vec::new();
        for &node_idx in &frontier {
            for edge in cpg.graph.edges(node_idx) {
                if *edge.weight() == Edge::Call {
                    let other = edge.target();
                    if !included.contains(&other.index()) {
                        included.insert(other.index());
                        next.push(other);
                    }
                }
            }
        }
        frontier = next;
        if included.len() >= max_nodes {
            break;
        }
    }
    let mut index_map = HashMap::new();
    let mut nodes = Vec::new();
    for &idx in &included {
        let new_idx = nodes.len();
        nodes.push(cpg.graph[petgraph::stable_graph::NodeIndex::new(idx)].clone());
        index_map.insert(idx, new_idx);
    }
    let mut edges = Vec::new();
    for &src_idx in &included {
        let src_node = petgraph::stable_graph::NodeIndex::new(src_idx);
        for edge in cpg.graph.edges(src_node) {
            let tgt_idx = edge.target().index();
            if included.contains(&tgt_idx) && *edge.weight() == Edge::Call {
                edges.push((
                    index_map[&src_idx],
                    index_map[&tgt_idx],
                    edge.weight().clone(),
                ));
            }
        }
    }
    GraphData { nodes, edges }
}

fn bench(c: &mut Criterion) {
    for &size in &[100, 500, 2000] {
        let cpg = make_test_graph(size);
        c.bench_function(&format!("subgraph_{}_nodes", size), |b| {
            b.iter(|| focal_graph_for_bench(black_box(&cpg), "func0", 50, 3))
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
