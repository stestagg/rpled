/// Macro to define a parser struct implementing ExtParser for an AST node
///
/// Usage:
/// ```
/// define_parser!(ParserName, AstNodeType, {
///     // parser implementation body
/// });
/// ```
#[macro_export]
macro_rules! parser {
    ($parser_name:ident($inp:ident) -> Result<$ast_type:ty> $body:block) => {
        pub struct $parser_name {}

        impl<'src, I, E> chumsky::extension::v1::ExtParser<'src, I, crate::ast::Spanned<$ast_type>, E> for $parser_name
        where
            I: chumsky::input::Input<'src, Token = u8>,
            E: chumsky::extra::ParserExtra<'src, I>,
        {
            fn parse(&self, $inp: &mut chumsky::input::InputRef<'src, '_, I, E>) -> Result<crate::ast::Spanned<$ast_type>, E::Error>
                $body
        }
    };
}

pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}


pub mod constant;
pub mod metadata;
pub mod program;
pub mod statement;
pub mod expr;
pub mod block;
pub mod call;

pub use constant::Constant;
pub use metadata::MetadataBlock;
pub use block::Block;