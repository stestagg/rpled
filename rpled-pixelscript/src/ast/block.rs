use super::statement::Statement;


#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    return_stmt: Option<Box<Statement>>,
}