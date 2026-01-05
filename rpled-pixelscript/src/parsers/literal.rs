use chumsky::prelude::*;
use chumsky::input::MapExtra;
use crate::ast::{Literal, Spanned};
use crate::lexer::Token;

/// Parser for literal values
pub fn literal<'src>() -> impl Parser<'src, &'src [Token], Spanned<Literal>, extra::Err<Rich<'src, Token>>> + Clone {
    select! {
        Token::Number(n) => Literal::Number(n),
        Token::Float(f) => Literal::Float(f),
        Token::String(s) => Literal::String(s.clone()),
        Token::True => Literal::Bool(true),
        Token::False => Literal::Bool(false),
        Token::Nil => Literal::Nil,
    }
    .map_with(|lit, e: &mut MapExtra<'src, '_, &'src [Token], _>| Spanned::new(lit, e.span().into_range()))
    .labelled("literal")
}
