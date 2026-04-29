use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    Cpp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Module,
    Function,
    Class,
    Variable,
    CallSite,
    Parameter,
    Import,
}

#[derive(Debug, thiserror::Error)]
pub enum IcbError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Graph error: {0}")]
    Graph(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
