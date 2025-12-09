
use chumsky::{
    prelude::*,
    input::InputRef,
    extension::v1::{ExtParser, Ext},
};
use crate::ast_format::{AstFormatInternal, FormattedAst};
use super::metadata::MetadataParser;
use super::block::BlockParser;


#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    metadata: MetadataBlock,
    block: Block,
}

pub struct ProgramParser{}

impl<'src, I, E> ExtParser<'src, I, Program, E> for ProgramParser
where
    I: Input<'src, Token = u8>,
    E: extra::ParserExtra<'src, I>,
{
    fn parse(&self, inp: &mut InputRef<'src, '_, I, E>) -> Result<Program, E::Error> {
        let metadata_block = Ext(MetadataParser{});
        let code_block = Ext(BlockParser{});

        let program_parser = metadata_block.then(code_block).map(|(metadata, block)| Program {
            metadata,
            block,
        });

        program_parser.parse(inp)
    }
}

// impl AstFormatInternal for Program {

//     fn format_internal(&self, output: &mut FormattedAst) {
//         ast_fmt!(output,
//             ["Program" blue], SP, '{',
//             (self.metadata), ',',
//             ["Block = " blue],(self.block),
//             '}',
//         )
        
//     }
// }
