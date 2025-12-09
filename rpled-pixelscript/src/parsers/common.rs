use chumsky::prelude::*;
use chumsky::input::{MapExtra, ValueInput};
use chumsky::span::SimpleSpan;
use crate::ast::Spanned;
use crate::lexer::Token;

/// Parser for identifiers
pub fn ident<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<String>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    select! {
        Token::Ident(s) => s.clone()
    }
    .map_with(|s, e: &mut MapExtra<'tokens, '_, I, _>| Spanned::new(s, e.span().into_range()))
    .labelled("identifier")
}

/// Parser for qualified names: Name or Name.Name.Name...
pub fn qualname<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<Vec<String>>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    ident()
        .separated_by(just(Token::Dot))
        .at_least(1)
        .collect()
        .map_with(|names: Vec<Spanned<String>>, ex: &mut MapExtra<'tokens, '_, I, _>| {
            let name_strings: Vec<String> = names.into_iter().map(|s| s.node).collect();
            Spanned::new(name_strings, ex.span().into_range())
        })
        .labelled("qualified name")
}
