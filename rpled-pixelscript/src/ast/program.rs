use super::prelude::*;
use super::metadata::MetadataBlock;


#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub metadata: MetadataBlock,
    pub block: Block,
}


parser!(for: Program {
    MetadataBlock::parser()
        .then_ignore(whitespace())
        .then(Block::parser())
        .map(|(metadata, block)| Program {
            metadata,
            block,
        })
});

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
