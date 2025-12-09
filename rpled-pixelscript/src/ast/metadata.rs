use std::collections::HashMap;
use crate::parsers::{call_parser, name_parser};
use super::prelude::*;


#[derive(Clone, Debug, PartialEq)]
pub enum MetadataValue {
    Constant(Constant),
    Nested(Box<MetadataTable>),
    List(Vec<Constant>),
    Call {name: String, args: Vec<MetadataValue>},
}

parser!(for: MetadataValue {
    recursive(|mv| {
        choice((
            // Try nested table first (starts with { and contains name = value pairs)
            MetadataTable::parser()
                .map(|table| MetadataValue::Nested(Box::new(table))),

            // Function call: name(arg1, arg2, ...)
            call_parser(mv)
                .map(|(name, args)| MetadataValue::Call {name, args}),

            // List of constants: { const, const, ... }
            Constant::parser()
                .then_ignore(whitespace())
                .separated_by(just(',').then(whitespace()))
                .allow_trailing()
                .collect::<Vec<_>>()
                .delimited_by(just('{').then(whitespace()), whitespace().then(just('}')))
                .map(MetadataValue::List),

            // Plain constant (fallback)
            Constant::parser()
                .map(MetadataValue::Constant),
        ))
    })
});


#[derive(Clone, Debug, PartialEq)]
pub struct MetadataTable {
    pub fields: HashMap<String, MetadataValue>,
}

parser!(for: MetadataTable {
    let separator = choice((
        newline().repeated().at_least(1).ignored(),
        just(';').then(whitespace()).ignored(),
    ));

    let field = name_parser()
        .then_ignore(whitespace())
        .then_ignore(just('='))
        .then_ignore(whitespace())
        .then(MetadataValue::parser());

    field
        .separated_by(separator)
        .allow_leading()
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(
            just('{').then(whitespace()),
            whitespace().then(just('}'))
        )
        .map(|fields| MetadataTable {
            fields: fields.into_iter().collect()
        })
});

#[derive(Clone, Debug, PartialEq)]
pub struct MetadataBlock(pub MetadataTable);

parser!(for: MetadataBlock {
    just("pixelscript")
        .then_ignore(just('='))
        .then(MetadataTable::parser())
        .map(|(_, table)| MetadataBlock(table))
});
    