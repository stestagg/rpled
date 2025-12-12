use crate::ast::block::Block;
use crate::ast::statement::Statement;
use crate::ast::NodeParser;
use chumsky::Parser;

#[test]
fn test_parse_empty_block() {
    let source = "";
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 0);
    } else {
        panic!("Expected empty block");
    }
}

#[test]
fn test_parse_single_statement() {
    let source = "x = 42";
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 1);
        if let Statement::Assignment { target, .. } = &statements[0] {
            assert_eq!(target, "x");
        } else {
            panic!("Expected assignment statement");
        }
    } else {
        panic!("Expected block");
    }
}

#[test]
fn test_parse_multiple_statements() {
    let source = r#"x = 42
y = 10
z = x + y"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 3);
    } else {
        panic!("Expected block with 3 statements");
    }
}

#[test]
fn test_parse_block_with_return() {
    let source = r#"x = 42
return x"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 2);
        matches!(statements[1], Statement::Return { .. });
    } else {
        panic!("Expected block with return statement");
    }
}

#[test]
fn test_parse_block_with_return_only() {
    let source = "return 42";
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 1);
        matches!(statements[0], Statement::Return { .. });
    } else {
        panic!("Expected block with single return statement");
    }
}

#[test]
fn test_parse_block_with_empty_return() {
    let source = r#"x = 42
return"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 2);
        if let Statement::Return { expr } = &statements[1] {
            assert!(expr.is_none());
        } else {
            panic!("Expected return statement");
        }
    } else {
        panic!("Expected block with empty return");
    }
}

#[test]
fn test_parse_block_with_function_calls() {
    let source = r#"foo(1, 2)
bar()
baz(x)"#;
    let result = Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 3);
        for stmt in statements {
            matches!(stmt, Statement::FunctionCall { .. });
        }
    } else {
        panic!("Expected block with function calls");
    }
}
