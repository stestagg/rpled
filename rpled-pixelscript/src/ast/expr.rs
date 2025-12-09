use super::prelude::*;
use crate::parsers::{call_parser, name_parser};


#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Constant(Constant),
    Variable(String),
    FunctionCall {name: String, args: Vec<Expression>},
    UnaryOp {op: String, expr: Box<Expression>},
    BinaryOp {left: Box<Expression>, op: String, right: Box<Expression>},
    TableDef(Vec<(Expression, Expression)>),
}

fn table_def_parser<'a>() -> impl Parser<'a, &'a str, Expression, Extra<'a>> + Clone {
    Expression::parser()
        .then_ignore(whitespace())
        .then_ignore(just('='))
        .then_ignore(whitespace())
        .then(Expression::parser())
        .separated_by(just(',').then(whitespace()))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(
            just('{').then(whitespace()),
            whitespace().then(just('}'))
        )
        .map(Expression::TableDef)
}

fn unary_op_parser<'a>() -> impl Parser<'a, &'a str, Expression, Extra<'a>> + Clone {
    choice((
        just('-').to_slice(),
        just("not").to_slice(),
    ))
    .map(|s: &str| s.to_string())
    .then_ignore(whitespace())
    .then(Expression::parser())
    .map(|(op, expr)| Expression::UnaryOp {
        op,
        expr: Box::new(expr),
    })
}

parser!(for: Expression {
    recursive(|expr| {
        choice((
            Constant::parser()
                .map(Expression::Constant),
            name_parser()
                .map(Expression::Variable),
            call_parser(expr)
                .map(|(name, args)| Expression::FunctionCall { name, args }),
            unary_op_parser(),
            // BinaryOp - TODO: need proper precedence handling
            table_def_parser(),
        ))
    })
});

