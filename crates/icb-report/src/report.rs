use icb_common::NodeKind;
use icb_graph::analysis;
use icb_graph::graph::{CodePropertyGraph, GraphData};
use serde_json::json;
use std::collections::HashSet;

/// Generates a full HTML report for the given project graph.
///
/// # Arguments
///
/// * `cpg` - The Code Property Graph to report on.
/// * `project_name` - A human‑readable name for the project.
///
/// # Returns
///
/// A `Result` containing the HTML string, or an error if serialization fails.
pub fn generate_report(
    cpg: &CodePropertyGraph,
    project_name: &str,
) -> Result<String, anyhow::Error> {
    let total_functions = cpg
        .graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::Function)
        .count();
    let total_classes = cpg
        .graph
        .node_weights()
        .filter(|n| n.kind == NodeKind::Class)
        .count();
    let total_calls = cpg
        .graph
        .edge_weights()
        .filter(|e| matches!(e, icb_graph::graph::Edge::Call))
        .count();

    let cycles = analysis::detect_call_cycles(cpg);
    let dead = analysis::detect_dead_code(cpg, &["main".to_string()]);
    let complex = analysis::detect_complex_functions(cpg, 20);

    let mut graph_data = GraphData::from(cpg);
    for node in &mut graph_data.nodes {
        let name = node.name.as_deref().unwrap_or("");
        let is_cycle = cycles
            .iter()
            .any(|c| c.functions.contains(&name.to_string()));
        let is_dead = dead.iter().any(|n| n.name.as_deref() == Some(name));
        let complexity = complex
            .iter()
            .find(|r| r.function_name == name)
            .map(|r| r.ast_node_count);
        node.usr = Some(
            json!({
                "is_cycle": is_cycle,
                "is_dead": is_dead,
                "complexity": complexity,
            })
            .to_string(),
        );
    }

    // Deduplicate edges (Graphology rejects duplicates)
    let mut seen = HashSet::new();
    graph_data.edges.retain(|(src, tgt, _)| {
        let key = (*src, *tgt);
        if seen.contains(&key) {
            false
        } else {
            seen.insert(key);
            true
        }
    });

    let json_graph = serde_json::to_string(&graph_data)?;
    let json_stats = json!({
        "project": project_name,
        "total_functions": total_functions,
        "total_classes": total_classes,
        "total_calls": total_calls,
        "cycle_count": cycles.len(),
        "dead_count": dead.len(),
        "complex_count": complex.len(),
    })
    .to_string();

    let html = include_str!("template_report.html")
        .replace("__GRAPH_DATA__", &json_graph)
        .replace("__STATS_DATA__", &json_stats)
        .replace("__PROJECT_NAME__", project_name);

    Ok(html)
}
