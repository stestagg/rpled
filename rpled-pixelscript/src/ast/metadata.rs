
#[derive(Clone, Debug, PartialEq)]
pub enum MetadataField {
    Literal(Literal),
    Nested(Box<MetadataTable>),
    List(Vec<Literal>),
    Call(String, Vec<Literal>)
}


#[derive(Clone, Debug, PartialEq)]
pub struct MetadataTable {
    pub fields: HashMap<String, MetadataField>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetadataBlock(pub MetadataTable);