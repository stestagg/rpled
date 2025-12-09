use super::literal::Literal;


#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    FunctionCall {name: String, args: Vec<Expression>},
    UnaryOp {op: String, expr: Box<Expression>},
    BinaryOp {left: Box<Expression>, op: String, right: Box<Expression>},
    TableDef(Vec<(Expression, Expression)>),
}


