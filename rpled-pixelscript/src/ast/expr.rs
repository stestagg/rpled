use super::Constant;


#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Constant(Constant),
    Variable(String),
    FunctionCall {name: String, args: Vec<Expression>},
    UnaryOp {op: String, expr: Box<Expression>},
    BinaryOp {left: Box<Expression>, op: String, right: Box<Expression>},
    TableDef(Vec<(Expression, Expression)>),
}


