use chumsky::prelude::*;
use chumsky::input::{MapExtra, ValueInput};
use chumsky::span::SimpleSpan;
use crate::ast::*;
use crate::lexer::Token;
use super::{common::{ident, qualname}, expr::expr};

/// Parser for statements
pub fn statement<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<Statement>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    recursive(|stmt| {
        choice((
            // break
            just(Token::Break)
                .to(Statement::Break)
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // local function definition: local function Name funcbody
            just(Token::Local)
                .ignore_then(just(Token::Function))
                .ignore_then(ident())
                .then(funcbody(stmt.clone()))
                .map(|(name, (params, block))| {
                    Statement::LocalFunctionDef(LocalFunctionDef { name, params, block })
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // local declaration: local Name [ = exp ]
            just(Token::Local)
                .ignore_then(ident())
                .then(just(Token::Assign).ignore_then(expr()).or_not())
                .map(|(name, value)| Statement::LocalDecl(LocalDecl { name, value }))
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // function definition: function Name funcbody
            just(Token::Function)
                .ignore_then(ident())
                .then(funcbody(stmt.clone()))
                .map(|(name, (params, block))| {
                    Statement::FunctionDef(FunctionDef { name, params, block })
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // for-in loop: for Name in Name do block end
            just(Token::For)
                .ignore_then(ident())
                .then_ignore(just(Token::In))
                .then(ident())
                .then_ignore(just(Token::Do))
                .then(block(stmt.clone()))
                .then_ignore(just(Token::End))
                .map(|((var, iterator), block)| {
                    Statement::ForIn(ForInStmt { var, iterator, block })
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // numeric for loop: for Name = exp, exp [, exp] do block end
            just(Token::For)
                .ignore_then(ident())
                .then_ignore(just(Token::Assign))
                .then(expr())
                .then_ignore(just(Token::Comma))
                .then(expr())
                .then(just(Token::Comma).ignore_then(expr()).or_not())
                .then_ignore(just(Token::Do))
                .then(block(stmt.clone()))
                .then_ignore(just(Token::End))
                .map(|((((var, start), end), step), block)| {
                    Statement::ForNum(ForNumStmt { var, start, end, step, block })
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // if statement: if exp then block { elseif exp then block } [ else block ] end
            just(Token::If)
                .ignore_then(expr())
                .then_ignore(just(Token::Then))
                .then(block(stmt.clone()))
                .then(
                    just(Token::Elseif)
                        .ignore_then(expr())
                        .then_ignore(just(Token::Then))
                        .then(block(stmt.clone()))
                        .repeated()
                        .collect::<Vec<_>>()
                )
                .then(
                    just(Token::Else)
                        .ignore_then(block(stmt.clone()))
                        .or_not()
                )
                .then_ignore(just(Token::End))
                .map(|(((condition, then_block), elseif_branches), else_block)| {
                    Statement::If(IfStmt {
                        condition,
                        then_block,
                        elseif_branches,
                        else_block,
                    })
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // repeat loop: repeat block until exp
            just(Token::Repeat)
                .ignore_then(block(stmt.clone()))
                .then_ignore(just(Token::Until))
                .then(expr())
                .map(|(block, condition)| {
                    Statement::Repeat(RepeatStmt { block, condition })
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // while loop: while exp do block end
            just(Token::While)
                .ignore_then(expr())
                .then_ignore(just(Token::Do))
                .then(block(stmt.clone()))
                .then_ignore(just(Token::End))
                .map(|(condition, block)| {
                    Statement::While(WhileStmt { condition, block })
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // do block: do block end
            just(Token::Do)
                .ignore_then(block(stmt.clone()))
                .then_ignore(just(Token::End))
                .map(|block| Statement::DoBlock(block))
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // Try to parse as function call or assignment
            // Both start with qualname, but only simple names can be assigned
            qualname()
                .then(choice((
                    // Assignment: Name = exp (only for simple names)
                    just(Token::Assign)
                        .ignore_then(expr())
                        .map(|value| AssignOrCall::Assign(value)),

                    // Function call: qualname args
                    expr()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .map(|args| AssignOrCall::Call(args)),
                )))
                .try_map(|(names, kind), span| {
                    match kind {
                        AssignOrCall::Assign(value) => {
                            // Only allow assignment to simple names
                            if names.node.len() == 1 {
                                Ok(Statement::Assignment(Assignment {
                                    var: Spanned::new(names.node[0].clone(), names.span),
                                    value
                                }))
                            } else {
                                Err(Rich::custom(span, "Cannot assign to qualified name"))
                            }
                        }
                        AssignOrCall::Call(args) => {
                            // Function calls can use both simple and qualified names
                            let prefix_expr = if names.node.len() == 1 {
                                PrefixExpr::Var(names.node[0].clone())
                            } else {
                                PrefixExpr::QualifiedName(names.node)
                            };
                            Ok(Statement::FunctionCall(FunctionCall {
                                func: Spanned::new(prefix_expr, names.span),
                                args,
                            }))
                        }
                    }
                })
                .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
                    Spanned::new(s, ex.span().into_range())
                }),
        ))
    })
}

/// Helper enum to distinguish between assignment and function call
enum AssignOrCall {
    Assign(Spanned<Expr>),
    Call(Vec<Spanned<Expr>>),
}

/// Parser for function body: ( [namelist] ) block end
fn funcbody<'tokens, 'src: 'tokens, I>(
    stmt: impl Parser<'tokens, I, Spanned<Statement>, extra::Err<Rich<'tokens, Token>>> + Clone + 'tokens,
) -> impl Parser<'tokens, I, (Vec<Spanned<String>>, Block), extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .then(block(stmt))
        .then_ignore(just(Token::End))
}

/// Parser for blocks: { stat [";"] } [ laststat [";"] ]
fn block<'tokens, 'src: 'tokens, I>(
    stmt: impl Parser<'tokens, I, Spanned<Statement>, extra::Err<Rich<'tokens, Token>>> + Clone + 'tokens,
) -> impl Parser<'tokens, I, Block, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    stmt.clone()
        .then_ignore(just(Token::Semicolon).or_not())
        .repeated()
        .collect()
        .then(return_stmt().then_ignore(just(Token::Semicolon).or_not()).or_not())
        .map(|(statements, return_stmt)| Block {
            statements,
            return_stmt: return_stmt.map(Box::new),
        })
}

/// Parser for return statement: return [exp]
fn return_stmt<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Spanned<ReturnStmt>, extra::Err<Rich<'tokens, Token>>> + Clone
where
    I: ValueInput<'tokens, Token = Token, Span = SimpleSpan>,
{
    just(Token::Return)
        .ignore_then(expr().or_not())
        .map(|value| ReturnStmt { value })
        .map_with(|s, ex: &mut MapExtra<'tokens, '_, I, _>| {
            Spanned::new(s, ex.span().into_range())
        })
}
