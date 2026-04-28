use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
}

#[derive(Debug, thiserror::Error)]
pub enum IcbError {
    #[error("Parsing error: {0}")]
    Parse(String),
    #[error("Graph error: {0}")]
    Graph(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}