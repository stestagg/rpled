use chumsky::prelude::*;
use chumsky::span::SimpleSpan;

use crate::ast::program::Program;
use chumsky::extension::v1::{ExtParser, Ext};
use crate::ast::{Spanned};



/// Parser for a complete pixelscript program: metadata block followed by code block
pub fn program<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<Program>, extra::Err<Rich<'tokens, ()>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    Ext(Program)
}