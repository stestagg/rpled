use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::ast::program::Program;
use chumsky::extension::v1::{ExtParser, Ext};
use crate::ast::{Spanned};


pub fn name_parser<'a>() -> impl Parser<'a, &'a str, Spanned<String>> {
    // Names are dot-separated identifiers:
    text::unicode::ident()
        .repeated()
        .at_least(1)
        .separated_by('.')
        .to_slice()
        .map_with_span(|s, span| Spanned::new(s.to_string(), span.start..span.end))
}



/// Parser for a complete pixelscript program: metadata block followed by code block
pub fn program<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<Program>, extra::Err<Rich<'tokens, ()>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    Ext(Program)
}