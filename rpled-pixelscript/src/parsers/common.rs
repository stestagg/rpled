use chumsky::prelude::*;
use chumsky::input::MapExtra;
use crate::ast::Spanned;
use crate::lexer::Token;

/// Parser for identifiers
pub fn ident<'src>() -> impl Parser<'src, &'src [Token], Spanned<String>, extra::Err<Rich<'src, Token>>> + Clone {
    select! {
        Token::Ident(s) => s.clone()
    }
    .map_with(|s, e: &mut MapExtra<'src, '_, &'src [Token], _>| Spanned::new(s, e.span().into_range()))
    .labelled("identifier")
}
