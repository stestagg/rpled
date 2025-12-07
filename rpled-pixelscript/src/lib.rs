#![feature(f16)]

pub mod ast;
pub mod lexer;
pub mod parsers;
pub mod parser;
pub mod error;
pub mod ast_format;

#[cfg(test)]
mod tests;

// Re-export commonly used types for convenience
pub use ast::{Program, Spanned, Expr, Statement, Block, Literal};
pub use lexer::{lex, Token};
pub use parser::program;
pub use error::{parse_program, format_lex_error, format_parse_errors};
