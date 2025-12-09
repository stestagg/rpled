use crate::{ast::prelude::*, parsers::{assignment_parser, call_parser, name_parser}};

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalBranch {
    pub condition: Expression,
    pub block: Box<Block>,
}

// Helper parser for if/elseif branches
fn conditional_branch_parser<'a>() -> impl Parser<'a, &'a str, ConditionalBranch, Extra<'a>> + Clone {
    Expression::parser()
        .then_ignore(whitespace())
        .then_ignore(just("then"))
        .then_ignore(whitespace())
        .then(Block::parser().boxed())
        .map(|(condition, block)| ConditionalBranch { condition, block: Box::new(block) })
}

fn if_parser<'a>() -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("if").ignored()
        .then(conditional_branch_parser())
        .then(
            just("elseif").ignored()
                .then_ignore(whitespace())
                .then(conditional_branch_parser())
                .map(|(_, branch)| branch)
                .repeated()
                .collect::<Vec<_>>()
        )
        .then(
            just("else").ignored()
                .then_ignore(whitespace())
                .then(Block::parser().boxed())
                .map(|(_, block)| block)
                .or_not()
        )
        .then_ignore(whitespace())
        .then_ignore(just("end"))
        .map(|(((_, if_part), else_if_part), else_part)| Statement::IfStmt {
            if_part,
            else_if_part,
            else_part: else_part.map(Box::new),
        })
}

fn for_in_parser<'a>() -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("for")
        .then_ignore(whitespace())
        .then(name_parser())
        .then_ignore(whitespace())
        .then_ignore(just("in"))
        .then_ignore(whitespace())
        .then(name_parser())
        .then_ignore(whitespace())
        .then(Block::parser().boxed())
        .map(|(((_, name), iter), block)| Statement::ForIn { name, iter, block: Box::new(block) })
}

fn for_num_parser<'a>() -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("for")
        .then_ignore(whitespace())
        .then(name_parser())
        .then_ignore(whitespace())
        .then_ignore(just('='))
        .then_ignore(whitespace())
        .then(Expression::parser())
        .then_ignore(whitespace())
        .then_ignore(just(','))
        .then_ignore(whitespace())
        .then(Expression::parser())
        .then(
            just(',').ignored()
                .then_ignore(whitespace())
                .then(Expression::parser())
                .map(|(_, expr)| expr)
                .or_not()
        )
        .then_ignore(whitespace())
        .then(Block::parser().boxed())
        .map(|(((((_, name), start), end), step), block)| Statement::ForNum {
            name,
            start,
            end,
            step,
            block: Box::new(block),
        })
}

fn function_def_parser<'a>() -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone{
    just("local")
        .or_not()
        .map(|v| v.is_some())
        .then_ignore(whitespace())
        .then_ignore(just("function"))
        .then_ignore(whitespace())
        .then(name_parser())
        .then(
            name_parser()
                .separated_by(just(','))
                .collect::<Vec<_>>()
                .delimited_by(just('('), just(')'))
        )
        .then_ignore(whitespace())
        .then(Block::parser().boxed())
        .map(|(((local, name), params), block)| Statement::FunctionDef {
            name,
            params,
            block: Box::new(block),
            local,
        })
}

fn while_parser<'a>() -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("while")
        .then_ignore(whitespace())
        .then(Expression::parser())
        .then_ignore(whitespace())
        .then(Block::parser().boxed())
        .map(|((_, cond), block)| Statement::WhileLoop { cond, block: Box::new(block) })
}

fn repeat_parser<'a>() -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone{
    just("repeat")
        .then_ignore(whitespace())
        .then(Block::parser().boxed())
        .then_ignore(whitespace())
        .then_ignore(just("until"))
        .then_ignore(whitespace())
        .then(Expression::parser())
        .map(|((_, block), cond)| Statement::RepeatLoop { cond, block: Box::new(block) })
}


#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment { target: String, value: Expression, local: bool },
    FunctionCall {name: String, args: Vec<Expression>},
    Block(Box<Block>),
    WhileLoop {cond: Expression, block: Box<Block>},
    RepeatLoop {cond: Expression, block: Box<Block>},
    IfStmt {if_part: ConditionalBranch, else_if_part: Vec<ConditionalBranch>, else_part: Option<Box<Block>>},
    ForIn {name: String, iter: String, block: Box<Block>},
    ForNum {name: String, start: Expression, end: Expression, step: Option<Expression>, block: Box<Block>},
    FunctionDef {name: String, params: Vec<String>, block: Box<Block>, local: bool}, 
}

parser!(for: Statement {
        choice((
            assignment_parser()
                .map(|(local, name, value)| Statement::Assignment { target: name, value, local }),
            call_parser(Expression::parser())
                .map(|(name, args)| Statement::FunctionCall { name, args }),
            Block::parser()
                .map(|block| Statement::Block(Box::new(block)))
                .boxed(),
            while_parser().boxed(),
            repeat_parser().boxed(),
            if_parser().boxed(),
            for_in_parser().boxed(),
            for_num_parser().boxed(),
            function_def_parser().boxed(),
        ))
    }
);