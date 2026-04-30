use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};
use clap::Parser;
use icb_common::Language;
use icb_graph::analysis;
use icb_graph::cache;
use icb_graph::graph::{CodePropertyGraph, Edge, GraphData};
use petgraph::visit::EdgeRef;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Parser)]
#[command(name = "icb-server")]
#[command(about = "ICB web server for graph visualization")]
struct Cli {
    #[arg(short, long)]
    project: PathBuf,
    #[arg(short, long, default_value = "python")]
    language: String,
    #[arg(long)]
    compile_commands: Option<PathBuf>,
    #[arg(long, default_value = "c++17")]
    cpp_std: String,
    #[arg(long)]
    cache: Option<PathBuf>,
    #[arg(short = 'P', long, default_value = "8080")]
    port: u16,
    #[arg(long, default_value = "web")]
    static_dir: PathBuf,
    #[arg(long)]
    no_system_headers: bool,
    #[arg(long)]
    max_depth: Option<usize>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Cli::parse();

    let graph = build_or_load_graph(
        &args.project,
        &args.language,
        args.compile_commands.as_ref(),
        &args.cpp_std,
        args.cache.as_ref(),
        args.no_system_headers,
        args.max_depth,
    )?;

    let graph_data = web::Data::new(Mutex::new(graph));
    let static_dir = args.static_dir.canonicalize().unwrap_or(args.static_dir);
    if !static_dir.exists() {
        eprintln!("Warning: static directory {:?} does not exist", static_dir);
    }
    println!("Serving static files from {:?}", static_dir);
    println!("Open http://localhost:{}", args.port);

    HttpServer::new(move || {
        App::new()
            .app_data(graph_data.clone())
            .route("/api/graph", web::get().to(get_graph))
            .route("/api/node", web::get().to(get_node_detail))
            .service(
                Files::new("/", static_dir.clone())
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await?;

    Ok(())
}

fn build_or_load_graph(
    project: &Path,
    language: &str,
    compile_commands: Option<&PathBuf>,
    cpp_std: &str,
    cache_path: Option<&PathBuf>,
    no_system_headers: bool,
    max_depth: Option<usize>,
) -> anyhow::Result<CodePropertyGraph> {
    let lang = parse_language(language)?;
    if let Some(cache_file) = cache_path {
        if cache_file.exists() {
            if let Ok(g) = cache::load_graph(cache_file) {
                return Ok(g);
            }
        }
    }

    let manager = icb_parser::manager::ParserManager::new();
    let allow_system = !no_system_headers;
    let file_facts: Vec<(String, Vec<icb_parser::facts::RawNode>)> = if lang == Language::Cpp {
        if let Some(cdb) = compile_commands {
            let cdb = cdb.canonicalize()?;
            let base_dir = cdb.parent().unwrap_or(Path::new("."));
            icb_clang::project::parse_project(&cdb, base_dir, true, allow_system)?
        } else if project.is_file() {
            let source = std::fs::read_to_string(project)?;
            let args = vec![format!("-std={}", cpp_std)];
            let facts = icb_clang::parser::parse_cpp_file(
                &source,
                &args,
                Some(project.to_str().unwrap_or("unknown")),
                allow_system,
            )?;
            vec![(
                project
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                facts,
            )]
        } else {
            let args = vec![format!("-std={}", cpp_std)];
            icb_clang::project::parse_directory(project, &args, true, max_depth, allow_system)?
        }
    } else if project.is_dir() {
        manager.parse_directory(lang, project)?
    } else {
        let source = std::fs::read_to_string(project)?;
        let facts = if lang == Language::Cpp {
            let args = vec![format!("-std={}", cpp_std)];
            icb_clang::parser::parse_cpp_file(
                &source,
                &args,
                Some(project.to_str().unwrap_or("unknown")),
                allow_system,
            )?
        } else {
            manager.parse_file(lang, &source)?
        };
        vec![(
            project
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            facts,
        )]
    };

    let mut builder = icb_graph::builder::GraphBuilder::new();
    for (_, facts) in file_facts {
        let mut local = icb_graph::builder::GraphBuilder::new();
        local.ingest_file_facts(&facts);
        builder.merge(local);
    }
    builder.resolve_calls();

    let cpg = builder.cpg;
    if let Some(cache_file) = cache_path {
        if let Err(e) = cache::save_graph(&cpg, cache_file) {
            log::warn!("Failed to save cache: {}", e);
        }
    }
    Ok(cpg)
}

fn parse_language(s: &str) -> anyhow::Result<Language> {
    match s {
        "python" => Ok(Language::Python),
        "rust" => Ok(Language::Rust),
        "javascript" => Ok(Language::JavaScript),
        "cpp" | "c++" => Ok(Language::Cpp),
        _ => anyhow::bail!("Unsupported language: {}", s),
    }
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

#[derive(serde::Serialize)]
struct NodeDetail {
    name: String,
    kind: String,
    line: usize,
    file: String,
    callers: Vec<String>,
    callees: Vec<String>,
    is_cycle: bool,
    is_dead: bool,
}

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

async fn get_node_detail(
    data: web::Data<Mutex<CodePropertyGraph>>,
    query: web::Query<std::collections::HashMap<String, String>>,
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

    let detail = NodeDetail {
        name: node.name.clone().unwrap_or_default(),
        kind: format!("{:?}", node.kind),
        line: node.start_line,
        file: node.usr.clone().unwrap_or_default(),
        callers,
        callees,
        is_cycle,
        is_dead,
    };
    HttpResponse::Ok().json(&detail)
}

fn find_node_index_by_name(cpg: &CodePropertyGraph, name: &str) -> usize {
    cpg.graph
        .node_indices()
        .find(|&idx| cpg.graph[idx].name.as_deref() == Some(name))
        .map(|idx| idx.index())
        .unwrap_or(usize::MAX)
}

/// Build a subgraph centered on `func_name` with neighbours up to `depth` hops.
fn focal_graph(
    cpg: &CodePropertyGraph,
    func_name: &str,
    max_nodes: usize,
    depth: usize,
) -> GraphData {
    let mut included = HashSet::new();
    let mut frontier = Vec::new();

    // Initial seed nodes
    for idx in cpg.graph.node_indices() {
        let node = &cpg.graph[idx];
        if (node.kind == icb_common::NodeKind::Function || node.kind == icb_common::NodeKind::Class)
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

    // Expand for `depth` hops
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

    // Limit total nodes
    if included.len() > max_nodes {
        let mut limited = HashSet::new();
        // Always keep the seed nodes
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
        Some("Function") => Some(icb_common::NodeKind::Function),
        Some("Class") => Some(icb_common::NodeKind::Class),
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
