use chumsky::prelude::*;
use chumsky::input::MapExtra;
use crate::ast::{MetadataBlock, MetadataField, MetadataTable, Spanned};
use crate::lexer::Token;
use super::{common::ident, literal::literal};

/// Parser for a single metadata field (with recursion support)
pub fn metadata_field<'src>() -> impl Parser<'src, &'src [Token], Spanned<MetadataField>, extra::Err<Rich<'src, Token>>> + Clone {
    recursive(|field| {
        ident()
            .then_ignore(just(Token::Assign))
            .then(choice((
                // Try call syntax first: name = (args)
                literal()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect()
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .map(|args| FieldValue::Call(args)),

                // We need to look ahead to distinguish between table and list
                // List: {literal, literal, ...} (no keys)
                // Table: {name = ..., ...} (has keys)
                // We'll parse the braces content and determine the type
                just(Token::LBrace)
                    .ignore_then(choice((
                        // Try to parse as list (no equals sign after first ident)
                        literal()
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect()
                            .map(|items| FieldValue::List(items)),
                        // Otherwise parse as table (recursive)
                        field
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect()
                            .map(|fields| {
                                let table = MetadataTable { fields };
                                FieldValue::TableFields(table)
                            }),
                    )))
                    .then_ignore(just(Token::RBrace)),

                // Finally literal: name = value
                literal().map(|lit| FieldValue::Literal(lit)),
            )))
            .map_with(|(name, value), e: &mut MapExtra<'src, '_, &'src [Token], _>| {
                let span = e.span().into_range();
                let field = match value {
                    FieldValue::Literal(lit) => MetadataField::Literal { name, value: lit },
                    FieldValue::TableFields(table) => {
                        // Need to wrap table in Spanned
                        let table_span = span.clone();
                        MetadataField::Table {
                            name,
                            table: Spanned::new(table, table_span),
                        }
                    }
                    FieldValue::Call(args) => MetadataField::Call { name, args },
                    FieldValue::List(items) => MetadataField::List { name, items },
                };
                Spanned::new(field, span)
            })
            .labelled("metadata field")
    })
}

/// Parser for metadata table
pub fn metadata_table<'src>() -> impl Parser<'src, &'src [Token], Spanned<MetadataTable>, extra::Err<Rich<'src, Token>>> + Clone {
    metadata_field()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .map(|fields| MetadataTable { fields })
        .map_with(|table, e: &mut MapExtra<'src, '_, &'src [Token], _>| Spanned::new(table, e.span().into_range()))
        .labelled("metadata table")
}

/// Parser for metadata block (the entire header)
pub fn metadata_block<'src>() -> impl Parser<'src, &'src [Token], Spanned<MetadataBlock>, extra::Err<Rich<'src, Token>>> + Clone {
    ident()
        .then_ignore(just(Token::Assign))
        .then(metadata_table())
        .map(|(name, table)| MetadataBlock { name, table })
        .map_with(|block, e: &mut MapExtra<'src, '_, &'src [Token], _>| Spanned::new(block, e.span().into_range()))
        .labelled("metadata block")
}

// Helper enum for discriminating field value types during parsing
enum FieldValue {
    Literal(Spanned<crate::ast::Literal>),
    TableFields(MetadataTable),
    Call(Vec<Spanned<crate::ast::Literal>>),
    List(Vec<Spanned<crate::ast::Literal>>),
}
