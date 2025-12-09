use anyhow::{anyhow, Context, Result};
use rpled_pixelscript::ast::{MetadataBlock, MetadataField, MetadataTable, Literal};
use crate::parser::ParsedProgram;

#[derive(Debug)]
pub struct PixelscriptHeader {
    pub name: String,
    pub modules: Vec<String>,
    pub entrypoint: String,
    // Store full params metadata for future use
    pub params: Option<MetadataTable>,
}

#[derive(Debug)]
pub struct ParsedScript {
    pub header: PixelscriptHeader,
    pub program: ParsedProgram,
}

impl ParsedScript {
    pub fn from_program(program: ParsedProgram) -> Result<Self> {
        log::debug!("Extracting pixelscript metadata");

        let header = PixelscriptHeader::from_metadata(program.metadata())
            .with_context(|| "Failed to extract pixelscript header")?;

        log::info!(
            "Found pixelscript: name='{}', modules={:?}, entrypoint='{}'",
            header.name,
            header.modules,
            header.entrypoint
        );

        Ok(Self { header, program })
    }
}

impl PixelscriptHeader {
    pub fn from_metadata(metadata: &MetadataBlock) -> Result<Self> {
        // Verify the metadata variable name is "pixelscript"
        if metadata.name.node != "pixelscript" {
            anyhow::bail!(
                "Expected metadata block named 'pixelscript', found '{}'",
                metadata.name.node
            );
        }

        let table = &metadata.table.node;

        let mut name = None;
        let mut modules = None;
        let mut entrypoint = None;
        let mut params = None;

        // Extract fields from metadata table
        for field_spanned in &table.fields {
            match &field_spanned.node {
                MetadataField::Literal { name: field_name, value } => {
                    match field_name.node.as_str() {
                        "name" => {
                            name = Some(extract_string_literal(&value.node)?);
                        }
                        "entrypoint" => {
                            entrypoint = Some(extract_string_literal(&value.node)?);
                        }
                        _ => {
                            log::warn!("Unknown metadata field: {}", field_name.node);
                        }
                    }
                }
                MetadataField::List { name: field_name, items } => {
                    match field_name.node.as_str() {
                        "modules" => {
                            modules = Some(extract_string_array(items)?);
                        }
                        _ => {
                            log::warn!("Unknown metadata list field: {}", field_name.node);
                        }
                    }
                }
                MetadataField::Table { name: field_name, table: nested_table } => {
                    match field_name.node.as_str() {
                        "params" => {
                            params = Some(nested_table.node.clone());
                        }
                        _ => {
                            log::warn!("Unknown metadata table field: {}", field_name.node);
                        }
                    }
                }
                MetadataField::Call { name: field_name, .. } => {
                    log::warn!("Ignoring metadata call field: {}", field_name.node);
                }
            }
        }

        Ok(PixelscriptHeader {
            name: name.ok_or_else(|| anyhow!("pixelscript.name is required"))?,
            modules: modules.ok_or_else(|| anyhow!("pixelscript.modules is required"))?,
            entrypoint: entrypoint.ok_or_else(|| anyhow!("pixelscript.entrypoint is required"))?,
            params,
        })
    }
}

// Helper functions for extracting literal values
fn extract_string_literal(lit: &Literal) -> Result<String> {
    match lit {
        Literal::String(s) => Ok(s.clone()),
        _ => anyhow::bail!("Expected string literal, got {:?}", lit),
    }
}

fn extract_string_array(items: &[rpled_pixelscript::Spanned<Literal>]) -> Result<Vec<String>> {
    items
        .iter()
        .map(|item| extract_string_literal(&item.node))
        .collect()
}
