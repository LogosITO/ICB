use actix_web::{web, App, HttpResponse, HttpServer};
use clap::Parser;
use icb_common::Language;
use icb_graph::cache;
use icb_graph::graph::{CodePropertyGraph, Edge, GraphData};
use petgraph::visit::EdgeRef;
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
    )?;

    let graph_data = web::Data::new(Mutex::new(graph));

    println!("Starting server at http://localhost:{}", args.port);
    HttpServer::new(move || {
        App::new()
            .app_data(graph_data.clone())
            .route("/", web::get().to(index_page))
            .route("/api/graph", web::get().to(get_graph))
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
) -> anyhow::Result<CodePropertyGraph> {
    let lang = parse_language(language)?;
    if let Some(cache_file) = cache_path {
        if cache_file.exists() {
            log::info!("Loading graph from cache {:?}", cache_file);
            if let Ok(g) = cache::load_graph(cache_file) {
                return Ok(g);
            }
        }
    }

    let manager = icb_parser::manager::ParserManager::new();
    let file_facts = if lang == Language::Cpp {
        if let Some(cdb) = compile_commands {
            let cdb = cdb.canonicalize()?;
            let base_dir = cdb.parent().unwrap_or(Path::new("."));
            icb_clang::project::parse_project(&cdb, base_dir, true)?
        } else if project.is_file() {
            let source = std::fs::read_to_string(project)?;
            let args = vec![format!("-std={}", cpp_std)];
            let facts = icb_clang::parser::parse_cpp_file(&source, &args)?;
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
            icb_clang::project::parse_directory(project, &args, true)?
        }
    } else if project.is_dir() {
        manager.parse_directory(lang, project)?
    } else {
        let source = std::fs::read_to_string(project)?;
        let facts = if lang == Language::Cpp {
            let args = vec![format!("-std={}", cpp_std)];
            icb_clang::parser::parse_cpp_file(&source, &args)?
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

async fn index_page() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(HTML_PAGE)
}

#[derive(serde::Deserialize)]
struct GraphQuery {
    kind: Option<String>,
    max_nodes: Option<usize>,
    focus: Option<String>,
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
    } = query.into_inner();

    let max = max_nodes.unwrap_or(200);

    let filtered = if let Some(func) = focus {
        focal_graph(&graph, &func, max)
    } else {
        subgraph_by_kind(&graph, kind.as_deref(), max)
    };

    HttpResponse::Ok().json(&filtered)
}

fn focal_graph(cpg: &CodePropertyGraph, func_name: &str, max_nodes: usize) -> GraphData {
    let mut included = HashSet::new();
    let mut selected_nodes = Vec::new();
    let mut selected_edges = Vec::new();

    let target_indices: Vec<usize> = cpg
        .graph
        .node_indices()
        .filter(|&idx| {
            let node = &cpg.graph[idx];
            (node.kind == icb_common::NodeKind::Function) && node.name.as_deref() == Some(func_name)
        })
        .map(|idx| idx.index())
        .collect();

    if target_indices.is_empty() {
        return GraphData {
            nodes: vec![],
            edges: vec![],
        };
    }

    for &tgt_idx in &target_indices {
        included.insert(tgt_idx);
    }

    for &tgt_idx in &target_indices {
        let tgt_node = petgraph::stable_graph::NodeIndex::new(tgt_idx);
        for edge_ref in cpg.graph.edges(tgt_node) {
            if *edge_ref.weight() == Edge::Call {
                included.insert(edge_ref.source().index());
                included.insert(edge_ref.target().index());
            }
        }
        for edge_ref in cpg
            .graph
            .edges_directed(tgt_node, petgraph::Direction::Incoming)
        {
            if *edge_ref.weight() == Edge::Call {
                included.insert(edge_ref.source().index());
                included.insert(edge_ref.target().index());
            }
        }
    }

    if included.len() > max_nodes {
        let mut limited = HashSet::new();
        for &tgt_idx in &target_indices {
            limited.insert(tgt_idx);
        }
        for &idx in &included {
            if limited.len() >= max_nodes {
                break;
            }
            if !limited.contains(&idx) {
                limited.insert(idx);
            }
        }
        included = limited;
    }

    let mut index_map = std::collections::HashMap::new();
    for &idx in &included {
        let node = &cpg.graph[petgraph::stable_graph::NodeIndex::new(idx)];
        let new_idx = selected_nodes.len();
        selected_nodes.push(node.clone());
        index_map.insert(idx, new_idx);
    }

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

const HTML_PAGE: &str = "<!DOCTYPE html>
<html>
<head>
<meta charset=\"utf-8\">
<title>ICB Graph</title>
<style>
  body { margin: 0; font-family: sans-serif; }
  #controls { padding: 10px; background: #f5f5f5; border-bottom: 1px solid #ccc; }
  #info { margin-left: 10px; display: inline; }
  svg { width: 100%; height: calc(100vh - 50px); }
  select { min-width: 200px; }
</style>
</head>
<body>
<div id=\"controls\">
  <label>Focus on function:</label>
  <select id=\"funcSelect\">
    <option value=\"\">-- all (limited) --</option>
  </select>
  <button id=\"focusBtn\">Focus</button>
  <span id=\"info\">Loading...</span>
</div>
<svg></svg>
<script src=\"https://d3js.org/d3.v7.min.js\"></script>
<script>
let allFunctions = [];

async function loadFunctions() {
  const resp = await fetch('/api/graph?kind=Function&max_nodes=500');
  const data = await resp.json();
  allFunctions = data.nodes;
  const select = document.getElementById('funcSelect');
  allFunctions.forEach(n => {
    const opt = document.createElement('option');
    opt.value = n.name || '';
    opt.textContent = n.name || '(anonymous)';
    select.appendChild(opt);
  });
}

async function updateGraph(focus = null) {
  document.getElementById('info').textContent = 'Loading...';
  const url = focus
    ? `/api/graph?focus=${encodeURIComponent(focus)}&max_nodes=200`
    : `/api/graph?kind=Function&max_nodes=150`;

  const resp = await fetch(url);
  const data = await resp.json();
  const nodes = data.nodes.map((n, i) => ({ id: i, label: n.name || '?', kind: n.kind }));
  const edges = data.edges.map(e => ({ source: e[0], target: e[1], kind: e[2] }));

  document.getElementById('info').textContent = `Nodes: ${nodes.length}, Edges: ${edges.length}`;

  const svg = d3.select(\"svg\");
  svg.selectAll(\"*\").remove();

  const width = window.innerWidth;
  const height = window.innerHeight - 50;

  const simulation = d3.forceSimulation(nodes)
    .force(\"link\", d3.forceLink(edges).id(d => d.id).distance(50))
    .force(\"charge\", d3.forceManyBody().strength(-200))
    .force(\"center\", d3.forceCenter(width/2, height/2));

  const link = svg.append(\"g\")
    .selectAll(\"line\")
    .data(edges)
    .join(\"line\")
    .attr(\"stroke\", \"#999\")
    .attr(\"stroke-opacity\", 0.6);

  const node = svg.append(\"g\")
    .selectAll(\"circle\")
    .data(nodes)
    .join(\"circle\")
    .attr(\"r\", 5)
    .attr(\"fill\", d => d.kind === \"Function\" ? \"steelblue\" : \"orange\")
    .call(drag(simulation));

  node.append(\"title\").text(d => `${d.label} (${d.kind})`);

  simulation.on(\"tick\", () => {
    link
      .attr(\"x1\", d => d.source.x)
      .attr(\"y1\", d => d.source.y)
      .attr(\"x2\", d => d.target.x)
      .attr(\"y2\", d => d.target.y);
    node
      .attr(\"cx\", d => d.x)
      .attr(\"cy\", d => d.y);
  });

  function drag(simulation) {
    function dragstarted(event, d) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      d.fx = d.x;
      d.fy = d.y;
    }
    function dragged(event, d) {
      d.fx = event.x;
      d.fy = event.y;
    }
    function dragended(event, d) {
      if (!event.active) simulation.alphaTarget(0);
      d.fx = null;
      d.fy = null;
    }
    return d3.drag()
      .on(\"start\", dragstarted)
      .on(\"drag\", dragged)
      .on(\"end\", dragended);
  }
}

document.getElementById('focusBtn').addEventListener('click', () => {
  const val = document.getElementById('funcSelect').value;
  updateGraph(val || null);
});

loadFunctions().then(() => updateGraph());
</script>
</body>
</html>";
