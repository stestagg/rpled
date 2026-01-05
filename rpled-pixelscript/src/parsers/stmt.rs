use chumsky::prelude::*;
use chumsky::input::MapExtra;
use crate::ast::*;
use crate::lexer::Token;
use super::{common::ident, expr::expr};

/// Parser for statements
pub fn statement<'src>() -> impl Parser<'src, &'src [Token], Spanned<Statement>, extra::Err<Rich<'src, Token>>> + Clone {
    recursive(|stmt| {
        choice((
            // break
            just(Token::Break)
                .to(Statement::Break)
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
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
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // local declaration: local Name [ = exp ]
            just(Token::Local)
                .ignore_then(ident())
                .then(just(Token::Assign).ignore_then(expr()).or_not())
                .map(|(name, value)| Statement::LocalDecl(LocalDecl { name, value }))
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // function definition: function Name funcbody
            just(Token::Function)
                .ignore_then(ident())
                .then(funcbody(stmt.clone()))
                .map(|(name, (params, block))| {
                    Statement::FunctionDef(FunctionDef { name, params, block })
                })
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
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
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
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
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
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
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
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
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
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
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // do block: do block end
            just(Token::Do)
                .ignore_then(block(stmt.clone()))
                .then_ignore(just(Token::End))
                .map(|block| Statement::DoBlock(block))
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
                    Spanned::new(s, ex.span().into_range())
                }),

            // Try to parse as function call or assignment
            // This is tricky because both start with Name
            // We need to look ahead to decide
            ident()
                .then(choice((
                    // Assignment: Name = exp
                    just(Token::Assign)
                        .ignore_then(expr())
                        .map(|value| AssignOrCall::Assign(value)),

                    // Function call: Name args
                    expr()
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect()
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .map(|args| AssignOrCall::Call(args)),
                )))
                .map(|(name, kind)| {
                    match kind {
                        AssignOrCall::Assign(value) => {
                            Statement::Assignment(Assignment { var: name, value })
                        }
                        AssignOrCall::Call(args) => {
                            Statement::FunctionCall(FunctionCall {
                                func: Spanned::new(PrefixExpr::Var(name.node.clone()), name.span.clone()),
                                args,
                            })
                        }
                    }
                })
                .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
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
fn funcbody<'src>(
    stmt: impl Parser<'src, &'src [Token], Spanned<Statement>, extra::Err<Rich<'src, Token>>> + Clone + 'src,
) -> impl Parser<'src, &'src [Token], (Vec<Spanned<String>>, Block), extra::Err<Rich<'src, Token>>> + Clone {
    ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .then(block(stmt))
        .then_ignore(just(Token::End))
}

/// Parser for blocks: { stat [";"] } [ laststat [";"] ]
fn block<'src>(
    stmt: impl Parser<'src, &'src [Token], Spanned<Statement>, extra::Err<Rich<'src, Token>>> + Clone + 'src,
) -> impl Parser<'src, &'src [Token], Block, extra::Err<Rich<'src, Token>>> + Clone {
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
fn return_stmt<'src>() -> impl Parser<'src, &'src [Token], Spanned<ReturnStmt>, extra::Err<Rich<'src, Token>>> + Clone {
    just(Token::Return)
        .ignore_then(expr().or_not())
        .map(|value| ReturnStmt { value })
        .map_with(|s, ex: &mut MapExtra<'src, '_, &'src [Token], _>| {
            Spanned::new(s, ex.span().into_range())
        })
}
