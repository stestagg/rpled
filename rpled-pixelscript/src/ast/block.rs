use super::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>
}

parser!(for: Block, recursing: statement: Statement {
    let returnstat = just("return").inlinepad()
        .then(Expression::parser().or_not())
        .map(|(_, expr)| Statement::Return { expr });

    statement
        .separated_by(one_of(";\n").inlinepad())
        .collect::<Vec<_>>()
        .map(|statements| {
            Block {
                statements
            }
        })
        .then(
            one_of(";\n").inlinepad()
            .ignore_then(returnstat)
            .or_not()
        )
        .map(|(mut block, ret)| {
            if let Some(ret_stmt) = ret {
                block.statements.push(ret_stmt);
            }
            block
        })
});

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