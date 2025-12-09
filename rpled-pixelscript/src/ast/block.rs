use super::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>
}

parser!(for: Block, recursing: statement: Statement {
    whitespace()
        .ignore_then(statement)
        .then_ignore(whitespace())
        .separated_by(one_of(";\n").then(whitespace()))
        .collect::<Vec<_>>()
        .map(|statements| Block {
            statements
        })
        .delimited_by(just('{'), just('}'))
    }
);

// Formatting implementation
impl AstFormat for Block {
    fn format_into(&self, f: &mut Formatter) {
        if self.statements.is_empty() {
            f.write("empty".dim());
        } else {
            f.write("{".green());
            f.list(&self.statements, |f, stmt| stmt.format_with_name(f));
            f.write("}".green());
        }
    }
}

impl AstFormatWithName for Block {
    const NODE_NAME: &'static str = "Block";
}