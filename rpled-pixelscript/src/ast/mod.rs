
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
    (for: $ast_type:ty, recursing: $($rec_name:ident: $rec_type:ty),* $body:block)  => {
        impl crate::ast::NodeParser for $ast_type {
            fn parser<'a>() -> impl chumsky::Parser<'a, &'a str, Self, crate::ast::Extra<'a>> + Clone
            where
                Self: Sized,
            {
                let $($rec_name),* = ($(<$rec_type>::parser()),*);
                $body
            }
        }
        // For each recursive paramter:
        $(impl $ast_type {
            paste::paste! {
                pub fn [<parser_with_ $rec_name>]< 'a>($rec_name: impl chumsky::Parser<'a, &'a str, $rec_type, crate::ast::Extra<'a>> + Clone + 'a) -> impl chumsky::Parser<'a, &'a str, Self, crate::ast::Extra<'a>> + Clone {
                    $body.labelled(stringify!($ast_type))
                }
            }
        })*
        // 

    }
}
// pub(crate) use parser;

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
    // pub(crate) use super::parser;
    pub use super::{NodeParser, Extra};

    // Parser extensions
    pub use crate::parser_ext::{InlinePadExt, inline_whitespace, comment, lineend};

    // Common AST nodes
    pub use super::constant::Constant;
    pub use super::expr::Expression;
    pub use super::statement::Statement;
    pub use super::block::Block;

    // Formatting
    pub use crate::format::{AstFormat, AstFormatWithName, Formatter};
    pub use yansi::Paint;
}