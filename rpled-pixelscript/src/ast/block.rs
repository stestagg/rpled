use super::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>
}

parser!(for: Block, recursing: statement: Statement {
    let returnstat = just("return").inlinepad()
        .then(Expression::parser().or_not())
        .labelled("return statement")
        .map(|(_, expr)| Statement::Return { expr });

    choice((
        statement
            .separated_by(lineend().then(inline_whitespace()).repeated())
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|statements| {
                println!("Parsed block with {} statements: {:?}", statements.len(), statements);
                Block {
                    statements
                }
            }).then(
                lineend().inlinepad().repeated().at_least(1)
                .ignore_then(returnstat.clone().map(|ret| {  
                    println!("Parsed return statement in block: {:?}", ret);
                    ret
                }))
                .or_not()
            )
            .map(|(mut block, ret)| {
                if let Some(ret_stmt) = ret {
                    block.statements.push(ret_stmt);
                }
                block
            }),
        returnstat.padded().map(|ret_stmt| {
            println!("Parsed block with only return statement: {:?}", ret_stmt);
            Block {
                statements: vec![ret_stmt]
            }
        }),
    ))
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