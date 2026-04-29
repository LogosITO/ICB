use icb_common::{Language, NodeKind};

/// Universal node representation independent of the parser.
#[derive(Debug, Clone)]
pub struct RawNode {
    pub language: Language,
    pub kind: NodeKind,
    pub name: Option<String>,
    pub usr: Option<String>,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub children: Vec<usize>,
}
