use chumsky::prelude::*;
use chumsky::input::{MapExtra, ValueInput};
use chumsky::span::SimpleSpan;
use crate::ast::{Literal, Spanned};
use crate::lexer::Token;

/// Parser for literal values
pub fn literal<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<Literal>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! {
        Token::Number(n) => Literal::Number(n),
        Token::Float(f) => Literal::Float(f),
        Token::String(s) => Literal::String(s.clone()),
        Token::True => Literal::Bool(true),
        Token::False => Literal::Bool(false),
        Token::Nil => Literal::Nil,
    }
    .map_with(|lit, e: &mut MapExtra<'tokens, '_, I, _>| Spanned::new(lit, e.span().into_range()))
    .labelled("literal")
}
