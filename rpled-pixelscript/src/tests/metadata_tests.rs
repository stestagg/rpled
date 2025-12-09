use crate::lexer::lex;
use crate::parsers::metadata::metadata_block;
use crate::ast::{MetadataField, Literal};
use chumsky::Parser;
use crate::tests::make_spanned_input;

#[test]
fn test_parse_simple_metadata() {
    let source = r#"pixelscript = { name = "Test" }"#;
    let tokens = lex(source).unwrap();
    let (result, errors) = metadata_block().parse(make_spanned_input(&tokens)).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some(), "No parse result");

    let block = result.unwrap();
    assert_eq!(block.node.name.node, "pixelscript");
    assert_eq!(block.node.table.node.fields.len(), 1);
}

#[test]
fn test_parse_metadata_with_multiple_fields() {
    let source = r#"pixelscript = { name = "Test", entrypoint = "main" }"#;
    let tokens = lex(source).unwrap();
    let (result, errors) = metadata_block().parse(make_spanned_input(&tokens)).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let block = result.unwrap();
    assert_eq!(block.node.table.node.fields.len(), 2);

    // Check first field
    if let MetadataField::Literal { name, value } = &block.node.table.node.fields[0].node {
        assert_eq!(name.node, "name");
        assert_eq!(value.node, Literal::String("Test".to_string()));
    } else {
        panic!("Expected Literal field");
    }
}

#[test]
fn test_parse_metadata_with_list() {
    let source = r#"pixelscript = { modules = {"LED", "TIME"} }"#;
    let tokens = lex(source).unwrap();
    let (result, errors) = metadata_block().parse(make_spanned_input(&tokens)).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let block = result.unwrap();

    // Check field is a list
    if let MetadataField::List { name, items } = &block.node.table.node.fields[0].node {
        assert_eq!(name.node, "modules");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].node, Literal::String("LED".to_string()));
        assert_eq!(items[1].node, Literal::String("TIME".to_string()));
    } else {
        panic!("Expected List field, got {:?}", block.node.table.node.fields[0].node);
    }
}

#[test]
fn test_parse_metadata_with_call() {
    let source = r#"config = { params = ("brightness", "speed") }"#;
    let tokens = lex(source).unwrap();
    let (result, errors) = metadata_block().parse(make_spanned_input(&tokens)).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let block = result.unwrap();

    // Check field is a call
    if let MetadataField::Call { name, args } = &block.node.table.node.fields[0].node {
        assert_eq!(name.node, "params");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].node, Literal::String("brightness".to_string()));
        assert_eq!(args[1].node, Literal::String("speed".to_string()));
    } else {
        panic!("Expected Call field");
    }
}

#[test]
fn test_parse_metadata_with_numbers() {
    let source = r#"config = { count = 10, rate = 3.14 }"#;
    let tokens = lex(source).unwrap();
    let (result, errors) = metadata_block().parse(make_spanned_input(&tokens)).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let block = result.unwrap();
    assert_eq!(block.node.table.node.fields.len(), 2);

    // Check number field
    if let MetadataField::Literal { name, value } = &block.node.table.node.fields[0].node {
        assert_eq!(name.node, "count");
        assert_eq!(value.node, Literal::Number(10));
    } else {
        panic!("Expected Literal field");
    }

    // Check float field
    if let MetadataField::Literal { name, value } = &block.node.table.node.fields[1].node {
        assert_eq!(name.node, "rate");
        assert_eq!(value.node, Literal::Float(3.14));
    } else {
        panic!("Expected Literal field");
    }
}

#[test]
fn test_parse_metadata_with_bools() {
    let source = r#"config = { enabled = true, debug = false }"#;
    let tokens = lex(source).unwrap();
    let (result, errors) = metadata_block().parse(make_spanned_input(&tokens)).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());
}
