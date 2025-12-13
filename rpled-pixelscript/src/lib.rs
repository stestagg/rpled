pub mod ast;
pub mod parsers;
pub mod parser_ext;
pub mod format;
pub mod error;

use ast::program::Program;
use chumsky::{Parser, ParseResult, error::Rich};
use ast::NodeParser as _;


#[cfg(test)]
mod tests;

pub fn parse_program<'a>(src: &'a str) -> ParseResult<Program, Rich<'a, char>>
{
    
    Program::parser().parse(src)
}