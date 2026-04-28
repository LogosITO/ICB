use tree_sitter::{Parser, Language as TsLanguage};
use icb_common::{Language, IcbError};

pub struct ParserManager {
    python_lang: TsLanguage,
}

impl ParserManager {
    pub fn new() -> Result<Self, IcbError> {
        Ok(ParserManager {
            python_lang: tree_sitter_python::language(),
        })
    }

    /// Теперь возвращает &TsLanguage, а не TsLanguage
    pub fn get_language(&self, lang: Language) -> &TsLanguage {
        match lang {
            Language::Python => &self.python_lang,
            _ => unimplemented!("язык пока не поддерживается"),
        }
    }

    pub fn parse(&self, lang: Language, source: &str) -> Result<tree_sitter::Tree, IcbError> {
        let mut parser = Parser::new();
        parser
            .set_language(self.get_language(lang))
            .map_err(|e| IcbError::Parse(e.to_string()))?;
        parser
            .parse(source, None)
            .ok_or_else(|| IcbError::Parse("Parsing returned None".into()))
    }
}