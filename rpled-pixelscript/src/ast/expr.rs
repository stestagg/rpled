use super::prelude::*;
use crate::parsers::{call_parser, name_parser};
use chumsky::pratt::{infix, prefix, left, right};


#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Constant(Constant),
    Variable(String),
    FunctionCall {name: String, args: Vec<Expression>},
    UnaryOp {op: String, expr: Box<Expression>},
    BinaryOp {left: Box<Expression>, op: String, right: Box<Expression>},
    TableDef(Vec<(Expression, Expression)>),
}

fn table_def_parser<'a>(expr: impl Parser<'a, &'a str, Expression, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Expression, Extra<'a>> + Clone {
    expr.clone()
        .then_ignore(just('=').inlinepad())
        .then(expr)
        .separated_by(just(',').inlinepad())
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(
            just('{').then(inline_whitespace()),
            inline_whitespace().then(just('}'))
        )
        .map(Expression::TableDef)
        .labelled("table definition")
}

fn binary_op_parser<'a>(expr: impl Parser<'a, &'a str, Expression, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Expression, Extra<'a>> + Clone {
    // Atom parsers - these are the base expressions
    let atom = choice((
        Constant::parser()
            .map(Expression::Constant),
        call_parser(expr.clone())
            .map(|(name, args)| Expression::FunctionCall { name, args }),
        name_parser()
            .map(Expression::Variable),
        table_def_parser(expr.clone()),
        expr.clone()
            .delimited_by(
                just('(').then(whitespace()),
                whitespace().then(just(')'))
            ),
    ));

    // Binary operators with precedence using Pratt parser
    atom.pratt((
        // Logical OR (lowest precedence)
        infix(left(1), just("or").inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "or".to_string(),
                right: Box::new(right),
            }
        }),
        // Logical AND
        infix(left(2), just("and").inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "and".to_string(),
                right: Box::new(right),
            }
        }),
        // Comparison operators
        infix(left(3), just("<=").inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "<=".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(3), just(">=").inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: ">=".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(3), just("==").inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "==".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(3), just("~=").inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "~=".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(3), just('<').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "<".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(3), just('>').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: ">".to_string(),
                right: Box::new(right),
            }
        }),
        // Addition and subtraction
        infix(left(5), just('+').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "+".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(5), just('-').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "-".to_string(),
                right: Box::new(right),
            }
        }),
        // Multiplication, division, modulo
        infix(left(6), just('*').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "*".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(6), just('/').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "/".to_string(),
                right: Box::new(right),
            }
        }),
        infix(left(6), just('%').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "%".to_string(),
                right: Box::new(right),
            }
        }),
        // Exponentiation (right-associative, highest precedence)
        infix(right(7), just('^').inlinepad(), |left, _, right, _| {
            Expression::BinaryOp {
                left: Box::new(left),
                op: "^".to_string(),
                right: Box::new(right),
            }
        }),
        // Unary operators (prefix)
        prefix(8, just('-').then(inline_whitespace()).to_slice(), |_, expr, _| {
            Expression::UnaryOp {
                op: "-".to_string(),
                expr: Box::new(expr),
            }
        }),
        prefix(8, just("not").then(inline_whitespace()).to_slice(), |_, expr, _| {
            Expression::UnaryOp {
                op: "not".to_string(),
                expr: Box::new(expr),
            }
        }),
    ))
}

parser!(for: Expression {
    recursive(|expr| {
        binary_op_parser(expr)
    })
});

// Formatting implementation
impl AstFormat for Expression {
    fn format_into(&self, f: &mut Formatter) {
        match self {
            // Delegate to child's format_with_name() for wrapped output
            Expression::Constant(c) => c.format_with_name(f),

            Expression::Variable(name) => {
                f.write("Variable".cyan());
                f.write_plain(": ");
                f.write(name.white());
            }

            Expression::BinaryOp { left, op, right } => {
                f.write("BinaryOp".cyan());
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
                f.write("UnaryOp".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.write(op.blue());
                    f.separator();
                    expr.format_with_name(f);
                });
            }

            Expression::FunctionCall { name, args } => {
                f.write("FunctionCall".cyan());
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
                f.write("TableDef".cyan());
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
