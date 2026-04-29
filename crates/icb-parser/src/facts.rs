use icb_common::{Language, NodeKind};

/// A single fact extracted from source code by a language parser.
///
/// `RawNode` is a language-agnostic representation of an AST node. It
/// captures the kind of the node, its location, an optional name, and
/// structural relationships through `children`.
///
/// # Example
///
/// ```rust
/// use icb_common::{Language, NodeKind};
/// use icb_parser::facts::RawNode;
///
/// let node = RawNode {
///     language: Language::Python,
///     kind: NodeKind::Function,
///     name: Some("hello".into()),
///     usr: None,
///     start_line: 1,
///     start_col: 0,
///     end_line: 1,
///     end_col: 10,
///     children: vec![],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RawNode {
    /// Programming language of the source file.
    pub language: Language,
    /// Kind of the node (function, class, call, etc.).
    pub kind: NodeKind,
    /// Optional human-readable name (e.g., function name).
    pub name: Option<String>,
    /// Optional unique identifier for cross-file references.
    pub usr: Option<String>,
    /// 1-based line where this node starts.
    pub start_line: usize,
    /// 0-based column where this node starts.
    pub start_col: usize,
    /// 1-based line where this node ends.
    pub end_line: usize,
    /// 0-based column where this node ends.
    pub end_col: usize,
    /// Indices of child `RawNode`s (in the same file) that are directly
    /// nested under this node according to the AST.
    pub children: Vec<usize>,
}
