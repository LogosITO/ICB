use crate::facts::RawNode;
use crate::lang::python::parse_python;
use icb_common::{IcbError, Language};

#[derive(Default)]
pub struct ParserManager;

impl ParserManager {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single file and return a flat list of facts.
    pub fn parse_file(&self, lang: Language, source: &str) -> Result<Vec<RawNode>, IcbError> {
        match lang {
            Language::Python => parse_python(source),
            _ => Err(IcbError::Parse(format!("unsupported language {:?}", lang))),
        }
    }
}
