use chumsky::prelude::*;
use chumsky::input::StrInput;
use chumsky::text::TextExpected;
use chumsky::label::LabelError;

/// Extension trait for parsers that adds inline padding (whitespace excluding newlines)
pub trait InlinePadExt<'a, I, O, E>: Parser<'a, I, O, E> + Sized + Clone
where
    I: Input<'a, Token = char> + StrInput<'a>,
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
    I: Input<'a, Token = char> + StrInput<'a>,
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
    I: Input<'a, Token = char> + StrInput<'a>,
    E: chumsky::extra::ParserExtra<'a, I>,
    P: Parser<'a, I, O, E> + Clone,
{
}

pub fn comment<'a, I, E>() -> impl Parser<'a, I, String, E> + Clone
where
    I: Input<'a, Token = char> + StrInput<'a>,
    E: chumsky::extra::ParserExtra<'a, I>,
{
    inline_whitespace()
        .ignore_then(just("--"))
        .ignore_then(inline_whitespace())
        .ignore_then(
            any().filter(|c| *c != '\n').repeated().collect::<String>()
        )
}

pub fn lineend<'a, I, E>() -> impl Parser<'a, I, (), E> + Clone
where
    I: Input<'a, Token = char> + StrInput<'a>,
    E: chumsky::extra::ParserExtra<'a, I>,
    E::Error: LabelError<'a, I, TextExpected<'a, I>>,
{
    comment().or_not().ignored().then_ignore(just('\r').or_not()).then_ignore(just('\n'))
    .or(
        just(';').ignore_then(comment().or_not().ignored()).then_ignore(text::whitespace())
    )
}