#[derive(Clone, Debug, PartialEq)]
pub struct Branch {
    condition: Expression,
    block: Block,
}


#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment { target: String, value: Expression, local: bool },
    FunctionCall {name: String, args: Vec<Expression>},
    Block(Block),
    WhileLoop {cond: Expression, block: Block},
    RepeatLoop {cond: Expression, block: Block},
    IfStmt {if_part: Branch, else_if_part: Vec<Branch>, else_part: Option<Block>},
    ForIn {name: String, iter: String, block: Block},
    ForNum {name: String, start: Expression, end: Expression, step: Option<Expression>, block: Block},
    FunctionDef {name: String, params: Vec<String>, block: Block, local: bool}, 
}