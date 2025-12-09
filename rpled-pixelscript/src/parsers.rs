use chumsky::prelude::*;
use chumsky::text::whitespace;

use crate::ast::{expr::Expression, program::Program};
use chumsky::extension::v1::{ExtParser, Ext};
use crate::ast::NodeParser as _;
use crate::ast::Extra;


pub fn name_parser<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> + Clone {
    // Names are dot-separated identifiers:
    text::unicode::ident::<_, Extra<'a>>()
        .repeated()
        .at_least(1)
        .separated_by(just('.'))
        .to_slice()
        .map(|s: &str| s.to_string())
}

pub fn assignment_parser<'a>() -> impl Parser<'a, &'a str, (bool, String, Expression), Extra<'a>> + Clone {
    just("local").or_not().map(|v| v.is_some())
        .then_ignore(whitespace())
        .then(name_parser())
        .then_ignore(whitespace())
        .then_ignore(just('='))
        .then_ignore(whitespace())
        .then(Expression::parser())
        .map(|((is_local, name), expr)| (is_local, name, expr))
}

pub fn call_parser<'a, T>(arg: impl Parser<'a, &'a str, T, Extra<'a>> + Clone) -> impl Parser<'a, &'a str, (String, Vec<T>), Extra<'a>> + Clone {
    name_parser()
    .then(arg
        .separated_by(just(','))
        .collect::<Vec<_>>()
        .delimited_by(just('('), just(')'))
    )
}

pub fn parse_program<'a>(src: &'a str) -> ParseResult<Program, Rich<'a, char>>
{
    Program::parser().parse(src)
}