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

// Formatting implementation
impl AstFormat for Expression {
    fn format_into(&self, f: &mut Formatter) {
        match self {
            // Delegate to child's format_with_name() for wrapped output
            Expression::Constant(c) => c.format_with_name(f),

            Expression::Variable(name) => {
                f.write("var".cyan());
                f.write_plain(": ");
                f.write(name.white());
            }

            Expression::BinaryOp { left, op, right } => {
                f.write("binop".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    left.format_with_name(f);
                    f.separator();
                    f.write(op.blue());
                    f.separator();
                    right.format_with_name(f);
                });
            }

            Expression::UnaryOp { op, expr } => {
                f.write("unop".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.write(op.blue());
                    f.separator();
                    expr.format_with_name(f);
                });
            }

            Expression::FunctionCall { name, args } => {
                f.write("call".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.write(name.white());
                    f.separator();
                    // Use curly braces for list (lisp-style)
                    f.write("{".green());
                    f.list(args, |f, arg| arg.format_with_name(f));
                    f.write("}".green());
                });
            }

            Expression::TableDef(fields) => {
                f.write("table".cyan());
                f.write_plain(": ");
                f.write("{".green());
                f.list(fields, |f, (key, value)| {
                    f.nested(|f| {
                        key.format_with_name(f);
                        f.write_plain(" ");
                        f.write("=>".cyan());
                        f.write_plain(" ");
                        value.format_with_name(f);
                    });
                });
                f.write("}".green());
            }
        }
    }
}

impl AstFormatWithName for Expression {
    const NODE_NAME: &'static str = "Expression";
}
