use std::collections::HashMap;

use chumsky::{prelude::*, extension::v1::Ext, extra, input::{Input, InputRef}};

use crate::{ast::{Constant, call::Call, constant::ConstantParser}, ast_format::{AstFormatInternal, FormattedAst}};


#[derive(Clone, Debug, PartialEq)]
pub enum MetadataValue {
    Constant(Constant),
    Nested(Box<MetadataTable>),
    List(Vec<Constant>),
    Call(Call)
}

crate::parser! {
    MetadataValueParser(inp) -> Result<MetadataValue> {
        // Implementation of the parser goes here
        choice(
            Ext(ConstantParser {}).map(MetadataValue::Constant),
            Ext(MetadataTableParser {}).map(|table| MetadataValue::Nested(Box::new(table))),
            Ext(ConstantParser {})
                .repeated()
                .map(MetadataValue::List)
                .delimited_by(just('{'), just('}')),
            Ext(CallParser {}).map(MetadataValue::Call)
        )
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct MetadataTable {
    pub fields: HashMap<String, MetadataValue>,
}

crate::parser! {
    MetadataTableParser(inp) -> Result<MetadataTable> {

        let parser = just('{')
            .then(whitespace())
            .then(name_parser())
            .then(whitespace())
            .then_ignore(just('='))
            .then(Ext(MetadataTableParser {}))
            .repeated()

        let field_parser = text::ident()
            .then_ignore(just('='))
            .then(Ext(MetadataValueParser {}))
            .then_ignore(just(';'));

        field_parser
            .repeated()
            .map(|fields: Vec<(String, MetadataField)>| {
                let mut map = HashMap::new();
                for (key, value) in fields {
                    map.insert(key, value);
                }
                MetadataTable { fields: map }
            })
            .delimited_by(just('{'), just('}'))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetadataBlock(pub MetadataTable);

impl AstFormatInternal for MetadataBlock {
    fn format_internal(&self, output: &mut FormattedAst) {
        let _ = output;
        crate::ast_fmt!(output,
            ["Meta" blue], ["[" cyan], (self.name.node), ["]" cyan], '=', (self.table)
        )
    }
}



crate::parser! {
    MetadataParser(inp) -> Result<MetadataBlock> {
        // Implementation of the parser goes here
        unimplemented!()
    }
}
