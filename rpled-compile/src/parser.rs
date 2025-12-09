use anyhow::{Context, Result};
use rpled_pixelscript::{parse_program, ast::{Program, MetadataBlock}, Spanned};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ParsedProgram {
    pub program: Spanned<Program>,
    pub source: String,
    pub filename: String,
}

impl ParsedProgram {
    pub fn from_file(path: &Path) -> Result<Self> {
        log::debug!("Reading file: {}", path.display());
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let filename = path.display().to_string();
        Self::from_source(&filename, source)
    }

    pub fn from_source(filename: &str, source: String) -> Result<Self> {
        log::debug!("Parsing {} bytes of pixelscript code", source.len());

        // Parse using rpled-pixelscript
        let program = parse_program(filename, &source)
            .map_err(|e| anyhow::anyhow!("Parse error:\n{}", e))?;

        log::info!("Successfully parsed pixelscript");
        log::debug!("Program statements: {}", program.node.block.statements.len());

        Ok(Self {
            program,
            source,
            filename: filename.to_string(),
        })
    }

    // Helper to access metadata
    pub fn metadata(&self) -> &MetadataBlock {
        &self.program.node.metadata.node
    }
}
