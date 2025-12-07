use anyhow::{Context, Result};
use luaparse::ast::Block;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ParsedLua<'a> {
    pub ast: Block<'a>,
    pub source: String,
}

impl<'a> ParsedLua<'a> {
    pub fn from_file(path: &Path) -> Result<ParsedLua<'static>> {
        log::debug!("Reading file: {}", path.display());
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        log::debug!("Parsing {} bytes of Lua code", source.len());

        // Leak the string to get a 'static lifetime
        let source_static = Box::leak(source.into_boxed_str());
        ParsedLua::from_source(source_static)
    }

    pub fn from_source(source: &'static str) -> Result<ParsedLua<'static>> {
        let ast = luaparse::parse(source).map_err(|e| {
            anyhow::anyhow!("Failed to parse pixelscript file: {}", e)
        })?;

        log::info!("Successfully parsed pixelscript");
        log::debug!("AST statements: {}", ast.statements.len());

        Ok(ParsedLua {
            ast,
            source: source.to_string(),
        })
    }
}
