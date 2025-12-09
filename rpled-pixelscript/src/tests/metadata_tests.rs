use crate::ast::metadata::{MetadataBlock, MetadataValue};
use crate::ast::constant::Constant;
use crate::ast::NodeParser;
use chumsky::Parser;

#[test]
fn test_parse_simple_metadata() {
    let source = r#"pixelscript = { name = "Test" }"#;
    let result = MetadataBlock::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let block = result.unwrap();
    assert_eq!(block.0.fields.len(), 1);
    assert!(block.0.fields.contains_key("name"));
}

#[test]
fn test_parse_metadata_with_multiple_fields() {
    let source = r#"pixelscript = { name = "Test", version = 1 }"#;
    let result = MetadataBlock::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let block = result.unwrap();
    assert_eq!(block.0.fields.len(), 2);

    // Check name field
    if let Some(MetadataValue::Constant(Constant::String(s))) = block.0.fields.get("name") {
        assert_eq!(s, "Test");
    } else {
        panic!("Expected name field with string value");
    }

    // Check version field
    if let Some(MetadataValue::Constant(Constant::Num(n))) = block.0.fields.get("version") {
        assert_eq!(*n, 1);
    } else {
        panic!("Expected version field with int value");
    }
}

#[test]
fn test_parse_metadata_with_list() {
    let source = r#"pixelscript = { modules = {"LED", "TIME"} }"#;
    let result = MetadataBlock::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let block = result.unwrap();

    // Check field is a list
    if let Some(MetadataValue::List(items)) = block.0.fields.get("modules") {
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], Constant::String("LED".to_string()));
        assert_eq!(items[1], Constant::String("TIME".to_string()));
    } else {
        panic!("Expected List field, got {:?}", block.0.fields.get("modules"));
    }
}

#[test]
fn test_parse_metadata_with_call() {
    let source = r#"pixelscript = { params = params("brightness", "speed") }"#;
    let result = MetadataBlock::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let block = result.unwrap();

    // Check field is a call
    if let Some(MetadataValue::Call { name, args }) = block.0.fields.get("params") {
        assert_eq!(name, "params");
        assert_eq!(args.len(), 2);
    } else {
        panic!("Expected Call field");
    }
}

#[test]
fn test_parse_metadata_with_numbers() {
    let source = r#"pixelscript = { count = 10, rate = 3.14 }"#;
    let result = MetadataBlock::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let block = result.unwrap();
    assert_eq!(block.0.fields.len(), 2);

    // Check number field
    if let Some(MetadataValue::Constant(Constant::Num(n))) = block.0.fields.get("count") {
        assert_eq!(*n, 10);
    } else {
        panic!("Expected Num field");
    }

    // Check float field
    if let Some(MetadataValue::Constant(Constant::Float(f))) = block.0.fields.get("rate") {
        assert_eq!(*f, 3.14);
    } else {
        panic!("Expected Float field");
    }
}

#[test]
fn test_parse_metadata_with_bools() {
    let source = r#"pixelscript = { enabled = true, debug = false }"#;
    let result = MetadataBlock::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let block = result.unwrap();

    // Check bool fields
    if let Some(MetadataValue::Constant(Constant::True)) = block.0.fields.get("enabled") {
        // Success
    } else {
        panic!("Expected True constant for enabled");
    }

    if let Some(MetadataValue::Constant(Constant::False)) = block.0.fields.get("debug") {
        // Success
    } else {
        panic!("Expected False constant for debug");
    }
}

#[test]
fn test_parse_metadata_with_nested_table() {
    let source = r#"pixelscript = {
        name = "nested",
        config = {
            fps = 60,
            brightness = 128
        }
    }"#;
    let result = MetadataBlock::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let block = result.unwrap();

    // Check nested table
    if let Some(MetadataValue::Nested(table)) = block.0.fields.get("config") {
        assert_eq!(table.fields.len(), 2);
        assert!(table.fields.contains_key("fps"));
        assert!(table.fields.contains_key("brightness"));
    } else {
        panic!("Expected Nested table");
    }
}
