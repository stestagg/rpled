use super::statement::Statement;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    name: String,
    args: Vec<Expression>,
}

crate::parser! {
    CallParser(inp) -> Result<Call> {
        // Placeholder implementation
        unimplemented!()
    }
}