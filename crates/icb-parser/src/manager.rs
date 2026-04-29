use crate::facts::RawNode;
use crate::lang::python::parse_python;
use icb_common::{IcbError, Language};

/// Central entry point for parsing source code.
///
/// `ParserManager` holds no state and simply dispatches to the appropriate
/// language-specific parser based on the requested [`Language`].
///
/// # Usage
///
/// ```rust
/// use icb_common::Language;
/// use icb_parser::manager::ParserManager;
///
/// let manager = ParserManager::new();
/// let facts = manager.parse_file(Language::Python, "def foo(): pass").unwrap();
/// assert!(!facts.is_empty());
/// ```
#[derive(Default)]
pub struct ParserManager;

impl ParserManager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self
    }

    /// Parse a single source file into a flat list of facts.
    ///
    /// # Errors
    ///
    /// Returns [`IcbError::Parse`] if the language is not supported or if
    /// the underlying parser fails.
    pub fn parse_file(&self, lang: Language, source: &str) -> Result<Vec<RawNode>, IcbError> {
        match lang {
            Language::Python => parse_python(source),
            _ => Err(IcbError::Parse(format!("unsupported language {:?}", lang))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_language_returns_error() {
        let manager = ParserManager::new();
        let res = manager.parse_file(Language::Rust, "fn main() {}");
        assert!(res.is_err());
        match res.unwrap_err() {
            IcbError::Parse(msg) => assert!(msg.contains("unsupported")),
            _ => panic!("Expected Parse error"),
        }
    }
}
