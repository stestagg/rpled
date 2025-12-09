
pub type Extra<'a> = chumsky::extra::Err<chumsky::error::Rich<'a, char>>;

pub trait NodeParser {
    fn parser<'a>() -> impl chumsky::Parser<'a, &'a str, Self, crate::ast::Extra<'a>> + Clone
    where
        Self: Sized;
}

macro_rules! parser {
    (for: $ast_type:ty $body:block) => {
        impl crate::ast::NodeParser for $ast_type {
            fn parser<'a>() -> impl chumsky::Parser<'a, &'a str, Self, crate::ast::Extra<'a>> + Clone
            where
                Self: Sized,
            $body
        }
    };
}
pub(crate) use parser;

pub mod constant;
pub mod metadata;
pub mod program;
pub mod statement;
pub mod expr;
pub mod block;

pub use constant::Constant;
pub use metadata::MetadataBlock;
pub use block::Block;

pub(crate) mod prelude {
    // Chumsky re-exports
    pub use chumsky::prelude::*;
    pub use chumsky::Parser;
    pub use chumsky::text::*;

    // Parser stuff
    pub(crate) use super::parser;
    pub use super::{NodeParser, Extra};

    // Common AST nodes
    pub use super::constant::Constant;
    pub use super::expr::Expression;
    pub use super::statement::Statement;
    pub use super::block::Block;
}