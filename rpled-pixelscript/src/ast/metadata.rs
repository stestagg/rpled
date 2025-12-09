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
            MetadataTable::parser_with_metadata_value(mv.clone())
                .map(|table| MetadataValue::Nested(Box::new(table))),

            // Function call: name(arg1, arg2, ...)
            call_parser(mv.clone())
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

parser!(for: MetadataTable, recursing: metadata_value: MetadataValue {
    let separator = choice((
        newline().repeated().at_least(1).ignored(),
        just(';').then(whitespace()).ignored(),
    ));

    let field = name_parser()
        .then_ignore(whitespace())
        .then_ignore(just('='))
        .then_ignore(whitespace())
        .then(metadata_value);

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

// Formatting implementations
impl AstFormat for MetadataValue {
    fn format_into(&self, f: &mut Formatter) {
        match self {
            MetadataValue::Constant(c) => c.format_with_name(f),

            MetadataValue::Nested(table) => table.format_with_name(f),

            MetadataValue::List(items) => {
                f.write("list".cyan());
                f.write_plain(": ");
                f.write("{".green());
                f.list(items, |f, item| item.format_with_name(f));
                f.write("}".green());
            }

            MetadataValue::Call { name, args } => {
                f.write("call".cyan());
                f.write_plain(": ");
                f.nested(|f| {
                    f.write(name.white());
                    f.separator();
                    f.write("{".green());
                    f.list(args, |f, arg| arg.format_with_name(f));
                    f.write("}".green());
                });
            }
        }
    }
}

impl AstFormatWithName for MetadataValue {
    const NODE_NAME: &'static str = "MetadataValue";
}

impl AstFormat for MetadataTable {
    fn format_into(&self, f: &mut Formatter) {
        f.write("{".green());
        let mut fields: Vec<_> = self.fields.iter().collect();
        fields.sort_by_key(|(k, _)| *k); // Sort for consistent output
        for (i, (key, value)) in fields.iter().enumerate() {
            f.field(key, |f| value.format_with_name(f));
            if i < fields.len() - 1 {
                f.comma();
            }
        }
        f.write("}".green());
    }
}

impl AstFormatWithName for MetadataTable {
    const NODE_NAME: &'static str = "MetadataTable";
}

impl AstFormat for MetadataBlock {
    fn format_into(&self, f: &mut Formatter) {
        self.0.format_with_name(f);
    }
}

impl AstFormatWithName for MetadataBlock {
    const NODE_NAME: &'static str = "MetadataBlock";
}
