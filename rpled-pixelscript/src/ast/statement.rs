use crate::{ast::prelude::*, parsers::{assignment_parser, call_parser, name_parser}};

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalBranch {
    pub condition: Expression,
    pub block: Box<Block>,
}

// Helper parser for if/elseif branches
fn conditional_branch_parser<'a>(statement: impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, ConditionalBranch, Extra<'a>> + Clone {
    Expression::parser()
        .then_ignore(just("then").inlinepad())
        .then(Block::parser_with_statement(statement).boxed())
        .map(|(condition, block)| ConditionalBranch { condition, block: Box::new(block) })
}

fn if_parser<'a>(statement: impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("if").ignored()
        .then(conditional_branch_parser(statement.clone()))
        .then(
            just("elseif").inlinepad()
                .ignore_then(conditional_branch_parser(statement.clone()))
                .repeated()
                .collect::<Vec<_>>()
        )
        .then(
            just("else").inlinepad()
                .ignore_then(Block::parser_with_statement(statement).boxed())
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

fn for_in_parser<'a>(statement: impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("for").inlinepad()
        .ignore_then(name_parser())
        .then_ignore(just("in").inlinepad())
        .then(name_parser())
        .then_ignore(just("do").inlinepad())
        .then(Block::parser_with_statement(statement).boxed())
        .then_ignore(whitespace())
        .then_ignore(just("end"))
        .map(|((name, iter), block)| Statement::ForIn { name, iter, block: Box::new(block) })
}

fn for_num_parser<'a>(statement: impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("for").inlinepad()
        .then(name_parser())
        .then_ignore(just('=').inlinepad())
        .then(Expression::parser())
        .then_ignore(just(',').inlinepad())
        .then(Expression::parser())
        .then(
            just(',').inlinepad()                
            .ignore_then(Expression::parser())
            .or_not()
        )
        .map(|((((_, name), start), end), step)| (name, start, end, step))
        .then_ignore(just("do").padded())
        
        .then(Block::parser_with_statement(statement).boxed())
        .then_ignore(whitespace())
        .then_ignore(just("end").padded())
        .map(|((name, start, end, step), block)| Statement::ForNum {
            name,
            start,
            end,
            step,
            block: Box::new(block),
        })
}

fn function_def_parser<'a>(statement: impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone{
    just("local")
        .or_not()
        .map(|v| v.is_some())
        .then_ignore(just("function").inlinepad())
        .then(name_parser())
        .then(
            name_parser()
                .separated_by(just(',').inlinepad())
                .collect::<Vec<_>>()
                .delimited_by(just('('), just(')'))
        )
        .then_ignore(whitespace())
        .then(Block::parser_with_statement(statement).boxed())
        .then_ignore(whitespace())
        .then_ignore(just("end"))
        .map(|(((local, name), params), block)| Statement::FunctionDef {
            name,
            params,
            block: Box::new(block),
            local,
        })
}

fn while_parser<'a>(statement: impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone {
    just("while").inlinepad()
        .ignore_then(Expression::parser())
        .then_ignore(just("do").padded())
        .then(Block::parser_with_statement(statement).boxed())        
        .then_ignore(just("end").padded())
        .map(|(cond, block)| Statement::WhileLoop { cond, block: Box::new(block) })
}

fn repeat_parser<'a>(statement: impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone + 'a) -> impl Parser<'a, &'a str, Statement, Extra<'a>> + Clone{
    just("repeat").inlinepad()
        .ignore_then(Block::parser_with_statement(statement).boxed())
        .then_ignore(just("until").padded())
        .then(Expression::parser())
        .map(|(block, cond)| Statement::RepeatLoop { cond, block: Box::new(block) })
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
    Return { expr: Option<Expression> },
}

parser!(for: Statement {
    recursive(|statement| {
        choice((
            assignment_parser()
                .map(|(local, name, value)| Statement::Assignment { target: name, value, local }),
            call_parser(Expression::parser())
                .map(|(name, args)| Statement::FunctionCall { name, args }),
            just("do").inlinepad()
                .ignore_then(Block::parser_with_statement(statement.clone()).boxed())
                .then_ignore(whitespace())
                .then_ignore(just("end"))
                .map(|block| Statement::Block(Box::new(block)))
                .boxed(),
            while_parser(statement.clone()).boxed(),
            repeat_parser(statement.clone()).boxed(),
            if_parser(statement.clone()).boxed(),
            for_in_parser(statement.clone()).boxed(),
            for_num_parser(statement.clone()).boxed(),
            function_def_parser(statement.clone()).boxed(),
        ))
    })
});

// Formatting implementation
impl AstFormat for Statement {
    fn format_into(&self, f: &mut Formatter) {
        match self {
            Statement::Assignment { target, value, local } => {
                f.write("assign".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    if *local {
                        f.write("local".magenta());
                        f.separator();
                    }
                    f.write(target.white());
                    f.separator();
                    value.format_with_name(f);
                });
            }

            Statement::FunctionCall { name, args } => {
                f.write("call".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.write(name.white());
                    f.separator();
                    f.write("{".green());
                    f.list(args, |f, arg| arg.format_with_name(f));
                    f.write("}".green());
                });
            }

            Statement::Block(block) => {
                block.format_with_name(f);
            }

            Statement::WhileLoop { cond, block } => {
                f.write("while".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.field("condition", |f| cond.format_with_name(f));
                    f.separator();
                    f.field("block", |f| block.format_with_name(f));
                });
            }

            Statement::RepeatLoop { cond, block } => {
                f.write("repeat".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.field("block", |f| block.format_with_name(f));
                    f.separator();
                    f.field("until", |f| cond.format_with_name(f));
                });
            }

            Statement::IfStmt { if_part, else_if_part, else_part } => {
                f.write("if".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.field("condition", |f| if_part.condition.format_with_name(f));
                    f.separator();
                    f.field("then", |f| if_part.block.format_with_name(f));

                    if !else_if_part.is_empty() {
                        f.separator();
                        f.field("elseif", |f| {
                            f.write("{".green());
                            f.list(else_if_part, |f, branch| {
                                f.nested(|f| {
                                    f.field("condition", |f| branch.condition.format_with_name(f));
                                    f.separator();
                                    f.field("then", |f| branch.block.format_with_name(f));
                                });
                            });
                            f.write("}".green());
                        });
                    }

                    if else_part.is_some() {
                        f.separator();
                        f.optional("else", else_part, |f, block| {
                            block.format_with_name(f);
                        });
                    }
                });
            }

            Statement::ForIn { name, iter, block } => {
                f.write("for-in".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.field("var", |f| f.write(name.white()));
                    f.separator();
                    f.field("in", |f| f.write(iter.white()));
                    f.separator();
                    f.field("block", |f| block.format_with_name(f));
                });
            }

            Statement::ForNum { name, start, end, step, block } => {
                f.write("for-num".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.field("var", |f| f.write(name.white()));
                    f.separator();
                    f.field("start", |f| start.format_with_name(f));
                    f.separator();
                    f.field("end", |f| end.format_with_name(f));
                    if step.is_some() {
                        f.separator();
                        f.optional("step", step, |f, s| s.format_with_name(f));
                    }
                    f.separator();
                    f.field("block", |f| block.format_with_name(f));
                });
            }

            Statement::FunctionDef { name, params, block, local } => {
                f.write("function".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    if *local {
                        f.write("local".magenta());
                        f.separator();
                    }
                    f.field("name", |f| f.write(name.white()));
                    f.separator();
                    f.field("params", |f| {
                        f.write("{".green());
                        f.list(params, |f, param| f.write(param.white()));
                        f.write("}".green());
                    });
                    f.separator();
                    f.field("block", |f| block.format_with_name(f));
                });
            }
            Statement::Return { expr } => {
                if let Some(expr) = expr {
                    f.nested(|f| {
                        expr.format_with_name(f);
                    });
                }
            }
        }
    }
}

impl AstFormatWithName for Statement {
    const NODE_NAME: &'static str = "Statement";
}

// Also implement for ConditionalBranch
impl AstFormat for ConditionalBranch {
    fn format_into(&self, f: &mut Formatter) {
        f.field("condition", |f| self.condition.format_with_name(f));
        f.separator();
        f.field("block", |f| self.block.format_with_name(f));
    }
}

impl AstFormatWithName for ConditionalBranch {
    const NODE_NAME: &'static str = "ConditionalBranch";
}