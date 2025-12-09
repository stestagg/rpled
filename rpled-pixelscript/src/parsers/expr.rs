use chumsky::prelude::*;
use chumsky::pratt::{left, right, prefix, infix};
use chumsky::input::{MapExtra, ValueInput};
use chumsky::span::SimpleSpan;
use crate::ast::{
    Expr, Spanned, BinaryOp, UnaryOp, BinOp, UnOp, PrefixExpr, IndexExpr,
    FunctionCall, TableConstructor, TableField,
};
use crate::lexer::Token;
use super::{common::{ident, qualname}, literal::literal};

/// Parser for expressions with proper operator precedence using Pratt parsing
pub fn expr<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<Expr>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|expr| {
        // Atom parsers: the basic building blocks of expressions
        let atom = choice((
            // Literals
            literal().map(|lit| lit.node).map(Expr::Literal),

            // Parenthesized expressions
            expr.clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(|e| Expr::Parenthesized(Box::new(e))),

            // Table constructors
            table_constructor().map(|tc| Expr::TableConstructor(tc.node)),

            // Variables and qualified names
            qualname().map(|qn| {
                if qn.node.len() == 1 {
                    Expr::Var(qn.node[0].clone())
                } else {
                    Expr::QualifiedName(qn.node)
                }
            }),
        ))
        .map_with(|e, ex: &mut MapExtra<'tokens, '_, I, _>| {
            Spanned::new(e, ex.span().into_range())
        });

        // Build prefix expressions (for indexing and function calls)
        let postfix_op = choice((
            // Indexing: expr[index]
            expr.clone()
                .delimited_by(just(Token::LBracket), just(Token::RBracket))
                .map(PostfixOp::Index),

            // Function call: expr(args)
            expr.clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(PostfixOp::Call),
        ));

        let prefix_expr = atom.clone().foldl(
            postfix_op.repeated(),
            |base: Spanned<Expr>, op: PostfixOp| {
                let acc = base;
                    let span = acc.span.clone();
                    match op {
                        PostfixOp::Index(index) => {
                            // Convert to PrefixExpr
                            let prefix_expr = match acc.node {
                                Expr::Var(name) => PrefixExpr::Var(name),
                                Expr::QualifiedName(parts) => PrefixExpr::QualifiedName(parts),
                                _ => {
                                    // Can't index non-name expressions
                                    return acc;
                                }
                            };
                            let end = index.span.end;
                            Spanned::new(
                                Expr::Index(IndexExpr {
                                    base: Box::new(Spanned::new(prefix_expr, span.clone())),
                                    index: Box::new(index),
                                }),
                                span.start..end,
                            )
                        }
                        PostfixOp::Call(args) => {
                            // Convert to PrefixExpr for function call
                            let prefix_expr = match acc.node {
                                Expr::Var(name) => PrefixExpr::Var(name),
                                Expr::QualifiedName(parts) => PrefixExpr::QualifiedName(parts),
                                _ => {
                                    // Can't call non-name expressions
                                    return acc;
                                }
                            };
                            let end = args.last().map(|a| a.span.end).unwrap_or(span.end);
                            Spanned::new(
                                Expr::FunctionCall(FunctionCall {
                                    func: Spanned::new(prefix_expr, span.clone()),
                                    args,
                                }),
                                span.start..end,
                            )
                        }
                    }
            },
        );

        // Pratt parser for binary and unary operators
        prefix_expr.pratt((
            // Unary prefix operators
            prefix(4, just(Token::Minus), |_op, rhs: Spanned<Expr>, _extra| {
                let span = rhs.span.clone();
                Spanned::new(
                    Expr::UnaryOp(UnaryOp {
                        op: Spanned::new(UnOp::Neg, span.clone()),
                        operand: Box::new(rhs),
                    }),
                    span,
                )
            }),
            prefix(4, just(Token::Not), |_op, rhs: Spanned<Expr>, _extra| {
                let span = rhs.span.clone();
                Spanned::new(
                    Expr::UnaryOp(UnaryOp {
                        op: Spanned::new(UnOp::Not, span.clone()),
                        operand: Box::new(rhs),
                    }),
                    span,
                )
            }),

            // Binary infix operators (lowest to highest precedence)
            infix(left(1), just(Token::Or), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                let span = lhs.span.start..rhs.span.end;
                let op_span = lhs.span.end..rhs.span.start;
                Spanned::new(
                    Expr::BinaryOp(BinaryOp {
                        left: Box::new(lhs),
                        op: Spanned::new(BinOp::Or, op_span),
                        right: Box::new(rhs),
                    }),
                    span,
                )
            }),

            infix(left(2), just(Token::And), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                let span = lhs.span.start..rhs.span.end;
                let op_span = lhs.span.end..rhs.span.start;
                Spanned::new(
                    Expr::BinaryOp(BinaryOp {
                        left: Box::new(lhs),
                        op: Spanned::new(BinOp::And, op_span),
                        right: Box::new(rhs),
                    }),
                    span,
                )
            }),

            // Comparison operators (all same precedence, left-associative)
            infix(left(3), just(Token::Lt), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Lt, rhs)
            }),
            infix(left(3), just(Token::Le), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Le, rhs)
            }),
            infix(left(3), just(Token::Gt), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Gt, rhs)
            }),
            infix(left(3), just(Token::Ge), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Ge, rhs)
            }),
            infix(left(3), just(Token::Eq), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Eq, rhs)
            }),
            infix(left(3), just(Token::Ne), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Ne, rhs)
            }),

            // Addition and subtraction
            infix(left(5), just(Token::Plus), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Add, rhs)
            }),
            infix(left(5), just(Token::Minus), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Sub, rhs)
            }),

            // Multiplication, division, modulo
            infix(left(6), just(Token::Star), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Mul, rhs)
            }),
            infix(left(6), just(Token::Slash), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Div, rhs)
            }),
            infix(left(6), just(Token::Percent), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Mod, rhs)
            }),

            // Exponentiation (right-associative, highest precedence)
            infix(right(7), just(Token::Caret), |lhs: Spanned<Expr>, _op, rhs: Spanned<Expr>, _extra| {
                make_binop(lhs, BinOp::Pow, rhs)
            }),
        ))
    })
}

/// Helper function to create binary operation expressions
fn make_binop(lhs: Spanned<Expr>, op: BinOp, rhs: Spanned<Expr>) -> Spanned<Expr> {
    let span = lhs.span.start..rhs.span.end;
    let op_span = lhs.span.end..rhs.span.start;
    Spanned::new(
        Expr::BinaryOp(BinaryOp {
            left: Box::new(lhs),
            op: Spanned::new(op, op_span),
            right: Box::new(rhs),
        }),
        span,
    )
}

/// Parser for table constructors
pub fn table_constructor<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<TableConstructor>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    table_field()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .map(|fields| TableConstructor { fields })
        .map_with(|tc, e: &mut MapExtra<'tokens, '_, I, _>| {
            Spanned::new(tc, e.span().into_range())
        })
        .labelled("table constructor")
}

/// Parser for table fields
fn table_field<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<TableField>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    choice((
        // [literal] = literal (indexed field)
        literal()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .then_ignore(just(Token::Assign))
            .then(literal())
            .map(|(key, value)| TableField::Indexed { key, value }),

        // name = literal (named field)
        ident()
            .then_ignore(just(Token::Assign))
            .then(literal())
            .map(|(name, value)| TableField::Named { name, value }),

        // literal (positional value)
        literal().map(|lit| TableField::Value(lit)),
    ))
    .map_with(|field, e: &mut MapExtra<'tokens, '_, I, _>| {
        Spanned::new(field, e.span().into_range())
    })
    .labelled("table field")
}

// Helper enum for postfix operations
enum PostfixOp {
    Index(Spanned<Expr>),
    Call(Vec<Spanned<Expr>>),
}
