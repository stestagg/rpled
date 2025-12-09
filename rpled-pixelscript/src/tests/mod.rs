mod lexer_tests;
mod metadata_tests;
mod expr_tests;
mod stmt_tests;
mod program_tests;
mod fixture_tests;
mod ast_format_tests;

use chumsky::prelude::*;
use crate::lexer::Token;
use crate::ast::Spanned;

/// Create a mapped input from lexed tokens with preserved spans
pub fn make_spanned_input(tokens: &[Spanned<Token>]) -> impl chumsky::input::ValueInput<'_, Token = Token, Span = SimpleSpan> + Clone {
    let end_span = tokens.last()
        .map(|t| t.span.end..t.span.end)
        .unwrap_or(0..0);

    tokens.map(end_span.into(), |spanned| (&spanned.node, SimpleSpan::from(spanned.span.clone())))
}
