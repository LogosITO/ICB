//! API route definitions and handlers for the ICB server.
//!
//! # Overview
//!
//! This module exposes a REST API consumed by the ICB dashboard.  All
//! routes are mounted under `/api` and operate on a shared
//! [`CodePropertyGraph`] that is built once at startup and held in an
//! `Arc<Mutex<…>>`.
//!
//! # Endpoints
//!
//! | Method | Path | Description |
//! |---|---|---|
//! | GET | `/api/graph` | Subgraph filtered by kind, focus, depth, max nodes, cycle/dead highlights |
//! | GET | `/api/node` | Detailed information about a single function |
//! | GET | `/api/functions` | All function metrics |
//! | GET | `/api/classes` | All class metrics |
//! | GET | `/api/files` | Per‑file aggregate metrics |
//! | GET | `/api/diff` | Compare two projects or cached graphs |
//!
//! # Diff endpoint
//!
//! The `/api/diff` endpoint accepts two mandatory query parameters, `old`
//! and `new`, which can be:
//!
//! * A path to a project directory or a single source file.
//! * A path to a previously cached `.icb` graph file.
//!
//! It returns a [`diff::DiffReport`] containing every node and edge
//! present in either graph, tagged as `Added`, `Removed`, or `Unchanged`.
//!
//! # Security
//!
//! In its current form the server is intended for local use.  The diff
//! endpoint can read arbitrary files reachable from the server process;
//! restrict network access appropriately.

use actix_web::{web, HttpResponse};
use icb_common::NodeKind;
use icb_graph::analysis;
use icb_graph::graph::{CodePropertyGraph, Edge, GraphData};
use petgraph::visit::EdgeRef;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;

use crate::analytics;
use crate::diff;
use crate::graph_builder;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/graph", web::get().to(get_graph))
            .route("/node", web::get().to(get_node_detail))
            .route("/functions", web::get().to(get_functions))
            .route("/classes", web::get().to(get_classes))
            .route("/files", web::get().to(get_files))
            .route("/diff", web::get().to(get_diff)),
    );
}

#[derive(Deserialize)]
struct GraphQuery {
    kind: Option<String>,
    max_nodes: Option<usize>,
    focus: Option<String>,
    depth: Option<usize>,
    show_cycles: Option<bool>,
    show_dead: Option<bool>,
    entries: Option<String>,
}

#[derive(Deserialize)]
struct DiffQuery {
    old: String,
    new: String,
    language: Option<String>,
}

/// Return a subgraph of the main CPG, filtered by the given parameters.
///
/// # Query parameters
///
/// * `kind` – node kind to include (`Function`, `Class`, etc.).
/// * `max_nodes` – maximum number of nodes in the response (default 200).
/// * `focus` – name of a function/class to start a focal expansion.
/// * `depth` – expansion depth when `focus` is used (default 1).
/// * `show_cycles` – annotate nodes that are part of a call cycle.
/// * `show_dead` – annotate nodes that are unreachable from the given
///   `entries` (default `"main"`).
/// * `entries` – comma‑separated list of entry point names.
async fn get_graph(
    data: web::Data<Mutex<CodePropertyGraph>>,
    query: web::Query<GraphQuery>,
) -> HttpResponse {
    let graph = data.lock().unwrap();
    let GraphQuery {
        kind,
        max_nodes,
        focus,
        depth,
        show_cycles,
        show_dead,
        entries,
    } = query.into_inner();
    let max = max_nodes.unwrap_or(200);
    let d = depth.unwrap_or(1);
    let show_cycles = show_cycles.unwrap_or(false);
    let show_dead = show_dead.unwrap_or(false);

    let filtered = if let Some(ref func) = focus {
        focal_graph(&graph, func, max, d)
    } else {
        subgraph_by_kind(&graph, kind.as_deref(), max)
    };

    if !show_cycles && !show_dead {
        return HttpResponse::Ok().json(&filtered);
    }

    let mut value = serde_json::to_value(&filtered).unwrap();
    if let Some(nodes) = value.get_mut("nodes").and_then(|n| n.as_array_mut()) {
        let cycle_nodes: HashSet<usize> = if show_cycles {
            let cycles = analysis::detect_call_cycles(&graph);
            cycles
                .iter()
                .flat_map(|c| &c.functions)
                .filter_map(|name| {
                    graph
                        .graph
                        .node_indices()
                        .find(|&idx| graph.graph[idx].name.as_deref() == Some(name))
                        .map(|idx| idx.index())
                })
                .collect()
        } else {
            HashSet::new()
        };

        let dead_nodes: HashSet<usize> = if show_dead {
            let entry_list: Vec<String> = entries
                .as_deref()
                .unwrap_or("main")
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            analysis::detect_dead_code(&graph, &entry_list)
                .iter()
                .filter_map(|node| {
                    graph
                        .graph
                        .node_indices()
                        .find(|&idx| graph.graph[idx].name.as_deref() == node.name.as_deref())
                        .map(|idx| idx.index())
                })
                .collect()
        } else {
            HashSet::new()
        };

        for node_val in nodes.iter_mut() {
            if let Some(obj) = node_val.as_object_mut() {
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                obj.insert(
                    "is_cycle".to_string(),
                    serde_json::Value::Bool(
                        cycle_nodes.contains(&find_node_index_by_name(&graph, &name)),
                    ),
                );
                obj.insert(
                    "is_dead".to_string(),
                    serde_json::Value::Bool(
                        dead_nodes.contains(&find_node_index_by_name(&graph, &name)),
                    ),
                );
            }
        }
    }

    HttpResponse::Ok().json(&value)
}

/// Return detailed information about a single function.
///
/// # Query parameters
///
/// * `name` – function name (required).
async fn get_node_detail(
    data: web::Data<Mutex<CodePropertyGraph>>,
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let graph = data.lock().unwrap();
    let name = match query.get("name") {
        Some(n) => n.clone(),
        None => return HttpResponse::BadRequest().json("missing 'name' parameter"),
    };

    let node_idx = match graph
        .graph
        .node_indices()
        .find(|&idx| graph.graph[idx].name.as_deref() == Some(&name))
    {
        Some(idx) => idx,
        None => return HttpResponse::NotFound().json("function not found"),
    };

    let node = &graph.graph[node_idx];
    let callers: Vec<String> = icb_graph::query::callers_of(&graph, &name)
        .iter()
        .map(|(n, _)| n.name.clone().unwrap_or_default())
        .collect();
    let callees: Vec<String> = icb_graph::query::callees_of(&graph, &name)
        .iter()
        .map(|(n, _)| n.name.clone().unwrap_or_default())
        .collect();
    let cycles = analysis::detect_call_cycles(&graph);
    let is_cycle = cycles.iter().any(|c| c.functions.contains(&name));
    let dead_entries = vec!["main".to_string()];
    let is_dead = analysis::detect_dead_code(&graph, &dead_entries)
        .iter()
        .any(|n| n.name.as_deref() == Some(&name));

    let detail = serde_json::json!({
        "name": node.name.clone().unwrap_or_default(),
        "kind": format!("{:?}", node.kind),
        "line": node.start_line,
        "file": node.usr.clone().unwrap_or_default(),
        "callers": callers,
        "callees": callees,
        "is_cycle": is_cycle,
        "is_dead": is_dead,
    });
    HttpResponse::Ok().json(&detail)
}

/// Return all function metrics (complexity, callers/callees, cycles, dead
/// code).
async fn get_functions(data: web::Data<Mutex<CodePropertyGraph>>) -> HttpResponse {
    let graph = data.lock().unwrap();
    let functions = analytics::collect_function_metrics(&graph);
    HttpResponse::Ok().json(&functions)
}

/// Return all class metrics (methods, complexity).
async fn get_classes(data: web::Data<Mutex<CodePropertyGraph>>) -> HttpResponse {
    let graph = data.lock().unwrap();
    let classes = analytics::collect_class_metrics(&graph);
    HttpResponse::Ok().json(&classes)
}

/// Return per‑file aggregate metrics (number of functions, classes, calls).
async fn get_files(data: web::Data<Mutex<CodePropertyGraph>>) -> HttpResponse {
    let graph = data.lock().unwrap();
    let files = analytics::collect_file_metrics(&graph);
    HttpResponse::Ok().json(&files)
}

/// Compare two projects or cached graphs and return a diff report.
///
/// # Query parameters
///
/// * `old` – path to the old project directory, source file, or `.icb`
///   cache file (required).
/// * `new` – path to the new project directory, source file, or `.icb`
///   cache file (required).
/// * `language` – programming language (default `"cpp"`).
/// * `no_system_headers` – exclude system header nodes (default `true`).
///
/// The response is a JSON object with `nodes` and `edges` arrays, each
/// element tagged with a `status` field (`"Added"`, `"Removed"`, or
/// `"Unchanged"`).
async fn get_diff(query: web::Query<DiffQuery>) -> HttpResponse {
    let lang = query.language.clone().unwrap_or_else(|| "cpp".into());

    let old_graph = graph_builder::build_or_load_graph(Path::new(&query.old), &lang, None);
    let new_graph = graph_builder::build_or_load_graph(Path::new(&query.new), &lang, None);

    match (old_graph, new_graph) {
        (Ok(old), Ok(new)) => HttpResponse::Ok().json(diff::diff_graphs(&old, &new)),
        (Err(e), _) | (_, Err(e)) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

fn find_node_index_by_name(cpg: &CodePropertyGraph, name: &str) -> usize {
    cpg.graph
        .node_indices()
        .find(|&idx| cpg.graph[idx].name.as_deref() == Some(name))
        .map(|idx| idx.index())
        .unwrap_or(usize::MAX)
}

fn focal_graph(
    cpg: &CodePropertyGraph,
    func_name: &str,
    max_nodes: usize,
    depth: usize,
) -> GraphData {
    let mut included = HashSet::new();
    let mut frontier = Vec::new();

    for idx in cpg.graph.node_indices() {
        let node = &cpg.graph[idx];
        if (node.kind == NodeKind::Function || node.kind == NodeKind::Class)
            && node.name.as_deref() == Some(func_name)
        {
            included.insert(idx.index());
            frontier.push(idx);
        }
    }

    if included.is_empty() {
        return GraphData {
            nodes: vec![],
            edges: vec![],
        };
    }

    for _ in 0..depth {
        let mut next_frontier = Vec::new();
        for &node_idx in &frontier {
            for edge_ref in cpg.graph.edges(node_idx) {
                if *edge_ref.weight() == Edge::Call {
                    let other = edge_ref.target();
                    if !included.contains(&other.index()) {
                        included.insert(other.index());
                        next_frontier.push(other);
                    }
                }
            }
            for edge_ref in cpg
                .graph
                .edges_directed(node_idx, petgraph::Direction::Incoming)
            {
                if *edge_ref.weight() == Edge::Call {
                    let other = edge_ref.source();
                    if !included.contains(&other.index()) {
                        included.insert(other.index());
                        next_frontier.push(other);
                    }
                }
            }
        }
        frontier = next_frontier;
        if included.len() >= max_nodes {
            break;
        }
    }

    if included.len() > max_nodes {
        let mut limited = HashSet::new();
        for &idx in &included {
            if limited.len() >= max_nodes {
                break;
            }
            limited.insert(idx);
        }
        included = limited;
    }

    let mut index_map = std::collections::HashMap::new();
    let mut selected_nodes = Vec::new();
    for &idx in &included {
        let node = &cpg.graph[petgraph::stable_graph::NodeIndex::new(idx)];
        let new_idx = selected_nodes.len();
        selected_nodes.push(node.clone());
        index_map.insert(idx, new_idx);
    }

    let mut selected_edges = Vec::new();
    for &src_idx in &included {
        let src_node = petgraph::stable_graph::NodeIndex::new(src_idx);
        for edge_ref in cpg.graph.edges(src_node) {
            let tgt_idx = edge_ref.target().index();
            if included.contains(&tgt_idx) && *edge_ref.weight() == Edge::Call {
                selected_edges.push((
                    index_map[&src_idx],
                    index_map[&tgt_idx],
                    edge_ref.weight().clone(),
                ));
            }
        }
    }

    GraphData {
        nodes: selected_nodes,
        edges: selected_edges,
    }
}

fn subgraph_by_kind(cpg: &CodePropertyGraph, kind: Option<&str>, max_nodes: usize) -> GraphData {
    let target_kind = match kind {
        Some("Function") => Some(NodeKind::Function),
        Some("Class") => Some(NodeKind::Class),
        _ => None,
    };

    let mut selected_nodes = Vec::new();
    let mut index_map = std::collections::HashMap::new();

    for idx in cpg.graph.node_indices() {
        if let Some(ref k) = target_kind {
            if cpg.graph[idx].kind != *k {
                continue;
            }
        }
        if selected_nodes.len() >= max_nodes {
            break;
        }
        let new_idx = selected_nodes.len();
        selected_nodes.push(cpg.graph[idx].clone());
        index_map.insert(idx.index(), new_idx);
    }

    let mut selected_edges = Vec::new();
    for idx in cpg.graph.node_indices() {
        if let Some(&mapped_src) = index_map.get(&idx.index()) {
            for edge_ref in cpg.graph.edges(idx) {
                let tgt_idx = edge_ref.target().index();
                if let Some(&mapped_tgt) = index_map.get(&tgt_idx) {
                    selected_edges.push((mapped_src, mapped_tgt, edge_ref.weight().clone()));
                }
            }
        }
    }

    GraphData {
        nodes: selected_nodes,
        edges: selected_edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use icb_graph::graph::{Edge, Node};

    fn test_graph() -> CodePropertyGraph {
        let mut cpg = CodePropertyGraph::new();
        let f1 = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("foo".into()),
            usr: Some("unit.cpp".into()),
            start_line: 1,
            end_line: 2,
        });
        let f2 = cpg.graph.add_node(Node {
            kind: NodeKind::Function,
            name: Some("bar".into()),
            usr: Some("unit.cpp".into()),
            start_line: 3,
            end_line: 4,
        });
        cpg.graph.add_edge(f1, f2, Edge::Call);
        cpg
    }

    #[actix_web::test]
    async fn test_get_functions() {
        let graph = test_graph();
        let data = web::Data::new(Mutex::new(graph));
        let app = test::init_service(App::new().app_data(data.clone()).configure(configure)).await;
        let req = test::TestRequest::get().uri("/api/functions").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
    }

    #[actix_web::test]
    async fn test_get_classes() {
        let graph = test_graph();
        let data = web::Data::new(Mutex::new(graph));
        let app = test::init_service(App::new().app_data(data.clone()).configure(configure)).await;
        let req = test::TestRequest::get().uri("/api/classes").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_files() {
        let graph = test_graph();
        let data = web::Data::new(Mutex::new(graph));
        let app = test::init_service(App::new().app_data(data.clone()).configure(configure)).await;
        let req = test::TestRequest::get().uri("/api/files").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
