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

// Formatting implementation
impl AstFormat for Program {
    fn format_into(&self, f: &mut Formatter) {
        f.field("metadata", |f| self.metadata.format_with_name(f));
        f.separator();
        f.field("block", |f| self.block.format_with_name(f));
    }
}

impl AstFormatWithName for Program {
    const NODE_NAME: &'static str = "Program";
}
