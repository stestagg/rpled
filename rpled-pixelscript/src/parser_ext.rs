use chumsky::prelude::*;
use chumsky::input::ValueInput;

/// Extension trait for parsers that adds inline padding (whitespace excluding newlines)
pub trait InlinePadExt<'a, I, O, E>: Parser<'a, I, O, E> + Sized + Clone
where
    I: Input<'a, Token = char> + ValueInput<'a>,
    E: chumsky::extra::ParserExtra<'a, I>,
{
    /// Pad this parser with inline whitespace on both sides.
    /// Unlike `.padded()`, this does not consume newlines.
    /// Consumes all unicode whitespace characters except '\n'.
    fn inlinepad(self) -> impl Parser<'a, I, O, E> + Clone {
        self.padded_by(inline_whitespace())
    }
}

/// Parser for inline whitespace (all unicode whitespace except newlines)
pub fn inline_whitespace<'a, I, E>() -> impl Parser<'a, I, (), E> + Clone
where
    I: Input<'a, Token = char> + ValueInput<'a>,
    E: chumsky::extra::ParserExtra<'a, I>,
{
    any()
        .filter(|c: &char| c.is_whitespace() && *c != '\n')
        .repeated()
        .ignored()
}

// Implement the extension for all parsers
impl<'a, I, O, E, P> InlinePadExt<'a, I, O, E> for P
where
    I: Input<'a, Token = char> + ValueInput<'a>,
    E: chumsky::extra::ParserExtra<'a, I>,
    P: Parser<'a, I, O, E> + Clone,
{
}
