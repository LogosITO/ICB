use icb_graph::graph::{CodePropertyGraph, Edge, GraphData};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde_json::json;
use std::collections::{HashMap, HashSet};

#[derive(serde::Serialize)]
struct DiffNode {
    name: String,
    kind: String,
    start_line: usize,
    status: String,
}

/// Generates an HTML diff page comparing two project graphs.
///
/// # Arguments
///
/// * `old_cpg` - The graph of the old project version.
/// * `new_cpg` - The graph of the new project version.
/// * `project_name` - A human‑readable name for the project.
///
/// # Returns
///
/// A `Result` containing the HTML string, or an error if serialization fails.
pub fn generate_diff(
    old_cpg: &CodePropertyGraph,
    new_cpg: &CodePropertyGraph,
    project_name: &str,
) -> Result<String, anyhow::Error> {
    let old_names: HashSet<&str> = old_cpg
        .graph
        .node_weights()
        .filter_map(|n| n.name.as_deref())
        .collect();
    let new_names: HashSet<&str> = new_cpg
        .graph
        .node_weights()
        .filter_map(|n| n.name.as_deref())
        .collect();

    let added: Vec<DiffNode> = new_names
        .difference(&old_names)
        .map(|name| {
            let node = new_cpg
                .graph
                .node_weights()
                .find(|n| n.name.as_deref() == Some(name))
                .unwrap();
            DiffNode {
                name: name.to_string(),
                kind: format!("{:?}", node.kind),
                start_line: node.start_line,
                status: "added".into(),
            }
        })
        .collect();

    let removed: Vec<DiffNode> = old_names
        .difference(&new_names)
        .map(|name| {
            let node = old_cpg
                .graph
                .node_weights()
                .find(|n| n.name.as_deref() == Some(name))
                .unwrap();
            DiffNode {
                name: name.to_string(),
                kind: format!("{:?}", node.kind),
                start_line: node.start_line,
                status: "removed".into(),
            }
        })
        .collect();

    let mut modified = Vec::new();
    let common = old_names.intersection(&new_names);
    for &name in common {
        let new_node = new_cpg
            .graph
            .node_weights()
            .find(|n| n.name.as_deref() == Some(name))
            .unwrap();
        let old_degree = old_cpg
            .graph
            .edges(
                old_cpg
                    .graph
                    .node_indices()
                    .find(|&i| old_cpg.graph[i].name.as_deref() == Some(name))
                    .unwrap(),
            )
            .filter(|e| matches!(e.weight(), Edge::Call))
            .count();
        let new_degree = new_cpg
            .graph
            .edges(
                new_cpg
                    .graph
                    .node_indices()
                    .find(|&i| new_cpg.graph[i].name.as_deref() == Some(name))
                    .unwrap(),
            )
            .filter(|e| matches!(e.weight(), Edge::Call))
            .count();
        if old_degree != new_degree {
            modified.push(DiffNode {
                name: name.to_string(),
                kind: format!("{:?}", new_node.kind),
                start_line: new_node.start_line,
                status: "modified".to_string(),
            });
        }
    }

    let mut graph = GraphData {
        nodes: vec![],
        edges: vec![],
    };
    let mut node_map = HashMap::new();

    for (idx, node) in new_cpg.graph.node_weights().enumerate() {
        let name = node.name.as_deref().unwrap_or("");
        let status = if removed.iter().any(|r| r.name == name) {
            "removed"
        } else if added.iter().any(|a| a.name == name) {
            "added"
        } else if modified.iter().any(|m| m.name == name) {
            "modified"
        } else {
            "unchanged"
        };
        let color = match status {
            "added" => "#4ade80",
            "removed" => "#f87171",
            "modified" => "#facc15",
            _ => "#60a5fa",
        };
        let extra = json!({ "status": status, "color": color });
        graph.nodes.push(icb_graph::graph::Node {
            kind: node.kind,
            name: node.name.clone(),
            usr: Some(extra.to_string()),
            start_line: node.start_line,
            end_line: node.end_line,
        });
        node_map.insert(name.to_string(), idx);
    }

    for edge_ref in new_cpg.graph.edge_references() {
        let src_name = new_cpg.graph[edge_ref.source()].name.clone();
        let tgt_name = new_cpg.graph[edge_ref.target()].name.clone();
        if let (Some(&src), Some(&tgt)) = (
            src_name.as_ref().and_then(|n| node_map.get(n)),
            tgt_name.as_ref().and_then(|n| node_map.get(n)),
        ) {
            graph.edges.push((src, tgt, edge_ref.weight().clone()));
        }
    }

    for node in old_cpg.graph.node_weights() {
        if let Some(name) = &node.name {
            if !node_map.contains_key(name) {
                let idx = graph.nodes.len();
                graph.nodes.push(icb_graph::graph::Node {
                    kind: node.kind,
                    name: node.name.clone(),
                    usr: Some(json!({ "status": "removed", "color": "#f87171" }).to_string()),
                    start_line: node.start_line,
                    end_line: node.end_line,
                });
                node_map.insert(name.clone(), idx);
            }
        }
    }

    let json_data = serde_json::to_string(&graph)?;
    let stats_json = json!({
        "added": added.len(),
        "removed": removed.len(),
        "modified": modified.len(),
    })
    .to_string();

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>ICB Diff - {project_name}</title>
  <style>
    :root {{ --bg: #0b0f17; --surface: #111827; --text: #e0e0e0; --accent: #60a5fa; }}
    body {{ font-family: 'Inter', system-ui, sans-serif; background: var(--bg); color: var(--text); margin: 0; }}
    .header {{ padding: 24px; background: var(--surface); border-bottom: 1px solid #1f2937; }}
    .header h1 {{ color: var(--accent); }}
    .stats {{ display: flex; gap: 24px; margin-top: 16px; flex-wrap: wrap; }}
    .stat {{ background: #1f2937; padding: 16px; border-radius: 12px; min-width: 120px; }}
    .stat .value {{ font-size: 2rem; font-weight: 700; }}
    .stat .label {{ color: #9ca3af; }}
    .legend {{ display: flex; gap: 16px; padding: 8px 16px; font-size: 0.875rem; }}
    .legend .dot {{ width: 10px; height: 10px; border-radius: 50%; display: inline-block; margin-right: 4px; }}
    #graph-container {{ height: 500px; margin: 16px; border-radius: 12px; overflow: hidden; }}
  </style>
</head>
<body>
  <div class="header">
    <h1>{project_name} – Diff</h1>
    <div class="stats" id="stats"></div>
  </div>
  <div class="legend">
    <span><span class="dot" style="background:#4ade80"></span> Added</span>
    <span><span class="dot" style="background:#f87171"></span> Removed</span>
    <span><span class="dot" style="background:#facc15"></span> Modified</span>
    <span><span class="dot" style="background:#60a5fa"></span> Unchanged</span>
  </div>
  <div id="graph-container"></div>

  <script src="https://cdn.jsdelivr.net/npm/graphology@0.25.4/dist/graphology.umd.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/graphology-layout-forceatlas2@0.9.1/dist/graphology-layout-forceatlas2.min.js"></script>
  <script src="https://cdn.jsdelivr.net/npm/sigma@2.4.0/build/sigma.min.js"></script>
  <script>
    const STATS_DATA = {stats_json};
    const GRAPH_DATA = {json_data};
  </script>
  <script>
    (function() {{
      document.getElementById('stats').innerHTML = [
        {{ value: STATS_DATA.added, label: 'Added' }},
        {{ value: STATS_DATA.removed, label: 'Removed' }},
        {{ value: STATS_DATA.modified, label: 'Modified' }}
      ].map(s => `<div class="stat"><div class="value">${{s.value}}</div><div class="label">${{s.label}}</div></div>`).join('');

      const g = new graphology.Graph();
      GRAPH_DATA.nodes.forEach((n, idx) => {{
        let extra = n.usr ? JSON.parse(n.usr) : {{}};
        g.addNode(idx, {{
          label: n.name || '?', kind: n.kind, line: n.start_line,
          x: Math.random() * 100, y: Math.random() * 100, size: 8,
          color: extra.color || '#60a5fa'
        }});
      }});
      GRAPH_DATA.edges.forEach(([src, tgt]) => {{
        if (src < GRAPH_DATA.nodes.length && tgt < GRAPH_DATA.nodes.length)
          g.addEdge(src, tgt, {{ size: 0.5, color: '#374151' }});
      }});

      const layout = new layoutForceAtlas2(g, {{ iterations: 100, settings: {{ gravity: 0.1, scalingRatio: 20 }} }});
      layout.start();
      setTimeout(() => layout.stop(), 2000);

      new Sigma(g, document.getElementById('graph-container'), {{
        renderLabels: false, minCameraRatio: 0.05, maxCameraRatio: 10
      }});
    }})();
  </script>
</body>
</html>"##,
        project_name = project_name,
        stats_json = stats_json,
        json_data = json_data,
    );

    Ok(html)
}
