use crate::graph::{CodePropertyGraph, GraphData};
use anyhow::Result;
use std::{fs, path::Path};

/// Load a cached graph from disk (bincode + zstd).
pub fn load_graph(path: &Path) -> Result<CodePropertyGraph> {
    let compressed = fs::read(path)?;
    let bytes = zstd::decode_all(&compressed[..])?;
    let data: GraphData = bincode::deserialize(&bytes)?;
    Ok(CodePropertyGraph::from(data))
}

/// Save a graph to disk (bincode + zstd).
pub fn save_graph(graph: &CodePropertyGraph, path: &Path) -> Result<()> {
    let data = GraphData::from(graph);
    let bytes = bincode::serialize(&data)?;
    let compressed = zstd::encode_all(&bytes[..], 0)?;
    fs::write(path, compressed)?;
    Ok(())
}
