use super::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    statements: Vec<Statement>
}

parser!(for: Block {
    whitespace()
        .ignore_then(Statement::parser())
        .then_ignore(whitespace())
        .separated_by(one_of(";\n").then(whitespace()))
        .collect::<Vec<_>>()
        .map(|statements| Block {
            statements
        })
        .delimited_by(just('{'), just('}'))
    }   
);