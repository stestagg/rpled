use crate::ast::block::Block;
use crate::ast::statement::Statement;
use crate::ast::NodeParser;
use chumsky::Parser;

#[test]
fn test_parse_empty_block() {
    let source = "";
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 0);
}

#[test]
fn test_parse_single_statement() {
    let source = "x = 42";
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 1);
    if let Statement::Assignment { target, .. } = &statements[0] {
        assert_eq!(target, "x");
    } else {
        panic!("Expected assignment statement");
    }
}

#[test]
fn test_parse_multiple_statements() {
    let source = r#"x = 42
y = 10
z = x + y"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 3);
}

#[test]
fn test_parse_block_with_return() {
    let source = r#"x = 42
return x"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 2);
    matches!(statements[1], Statement::Return { .. });
}

#[test]
fn test_parse_block_with_return_only() {
    let source = "return 42";
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 1);
    matches!(statements[0], Statement::Return { .. });
}

#[test]
fn test_parse_block_with_empty_return() {
    let source = r#"x = 42
return"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 2);
    if let Statement::Return { expr } = &statements[1] {
        assert!(expr.is_none());
    } else {
        panic!("Expected return statement");
    }
}

#[test]
fn test_parse_block_with_function_calls() {
    let source = r#"foo(1, 2)
bar()
baz(x)"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 3);
    for stmt in statements {
        matches!(stmt, Statement::FunctionCall { .. });
    }
}
