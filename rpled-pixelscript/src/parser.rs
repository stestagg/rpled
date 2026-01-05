use chumsky::prelude::*;
use chumsky::input::MapExtra;
use crate::ast::{Program, MetadataBlock, Block, Spanned};
use crate::lexer::Token;
use crate::parsers::{common::ident, metadata::metadata_table, stmt::statement};

/// Parser for a complete pixelscript program: metadata block followed by code block
pub fn program<'src>() -> impl Parser<'src, &'src [Token], Spanned<Program>, extra::Err<Rich<'src, Token>>> + Clone {
    metadata_block()
        .then(block())
        .map_with(|(metadata, block), ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
            let span = ex.span().into_range();
            Spanned::new(
                Program {
                    metadata,
                    block,
                    span: span.clone(),
                },
                span,
            )
        })
}

/// Parser for metadata block: Name = metadata_table
fn metadata_block<'src>() -> impl Parser<'src, &'src [Token], Spanned<MetadataBlock>, extra::Err<Rich<'src, Token>>> + Clone {
    ident()
        .then_ignore(just(Token::Assign))
        .then(metadata_table())
        .map(|(name, table)| MetadataBlock { name, table })
        .map_with(|mb, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
            Spanned::new(mb, ex.span().into_range())
        })
}

/// Parser for blocks: { stat [";"] } [ laststat [";"] ]
fn block<'src>() -> impl Parser<'src, &'src [Token], Block, extra::Err<Rich<'src, Token>>> + Clone {
    statement()
        .then_ignore(just(Token::Semicolon).or_not())
        .repeated()
        .collect()
        .then(return_stmt().then_ignore(just(Token::Semicolon).or_not()).or_not())
        .map(|(statements, return_stmt)| Block {
            statements,
            return_stmt: return_stmt.map(Box::new),
        })
}

/// Parser for return statement: return [exp]
fn return_stmt<'src>() -> impl Parser<'src, &'src [Token], Spanned<crate::ast::ReturnStmt>, extra::Err<Rich<'src, Token>>> + Clone {
    use crate::parsers::expr::expr;

    just(Token::Return)
        .ignore_then(expr().or_not())
        .map(|value| crate::ast::ReturnStmt { value })
        .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
            Spanned::new(s, ex.span().into_range())
        })
}
