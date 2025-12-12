use crate::ast::statement::Statement;
use crate::ast::{Block, NodeParser};
use chumsky::Parser;
use crate::format::AstFormat;


#[test]
fn test_parse_assignment() {
    let source = "x = 42";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::Assignment { target, local, .. } = result.unwrap() {
        assert_eq!(target, "x");
        assert_eq!(local, false);
    } else {
        panic!("Expected assignment");
    }
}

#[test]
fn test_parse_consecutive_assignment() {
    let source = r#"x = 42
x = 12
"#;
    let result = crate::ast::block::Block::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Block { statements } = result.unwrap() {
        assert_eq!(statements.len(), 2);
        if let Statement::Assignment { target, .. } = &statements[0] {
            assert_eq!(target, "x");
        } else {
            panic!("Expected assignment");
        }
    } else {
        panic!("Expected assignment");
    }
}


#[test]
fn test_parse_local_assignment() {
    let source = "local x = 42";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::Assignment { target, local, .. } = result.unwrap() {
        assert_eq!(target, "x");
        assert_eq!(local, true);
    } else {
        panic!("Expected local assignment");
    }
}

#[test]
fn test_parse_function_call_statement() {
    let source = "foo(1, 2)";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::FunctionCall { name, args } = result.unwrap() {
        assert_eq!(name, "foo");
        assert_eq!(args.len(), 2);
    } else {
        panic!("Expected function call statement");
    }
}

#[test]
fn test_parse_qualified_function_call() {
    let source = "led.set(1, 255)";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::FunctionCall { name, args } = result.unwrap() {
        assert_eq!(name, "led.set");
        assert_eq!(args.len(), 2);
    } else {
        panic!("Expected qualified function call");
    }
}

#[test]
fn test_parse_while_loop() {
    let source = r#"
while sum < 100 do
    foo = 1
end
"#;
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[test]
fn test_parse_if_statement() {
    let source = "if x > 5 then y = 1 end";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::IfStmt { if_part, else_if_part, else_part } = result.unwrap() {
        assert!(else_if_part.is_empty());
        assert!(else_part.is_none());
    } else {
        panic!("Expected if statement");
    }
}

#[test]
fn test_parse_if_else_statement() {
    let source = "if x > 5 then y = 1 else y = 0 end";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::IfStmt { else_part, .. } = result.unwrap() {
        assert!(else_part.is_some());
    } else {
        panic!("Expected if-else statement");
    }
}

// if i % 15 == 0 then
#[test]
fn test_if_complex_cond() {
    let source = "if i % 15 == 0 then y = 15 else y = 0 end";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::IfStmt { if_part, .. } = result.unwrap() {
        assert_eq!(if_part.condition.compact_plain_format(), "BinaryOp: [Expression:[BinaryOp: [Expression:[Variable: i] % Expression:[Constant:[Num: 15]]]] == Expression:[Constant:[Num: 0]]]");
    } else {
        panic!("Expected if-else statement");
    }
}

#[test]
fn test_parse_if_elseif_statement() {
    let source = "if x > 10 then y = 2 elseif x > 5 then y = 1 else y = 0 end";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::IfStmt { else_if_part, else_part, .. } = result.unwrap() {
        assert_eq!(else_if_part.len(), 1);
        assert!(else_part.is_some());
    } else {
        panic!("Expected if-elseif-else statement");
    }
}

#[test]
fn test_parse_for_numeric() {
    let source = "for i = 1, 10 sum = sum + i";
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[test]
fn test_parse_for_numeric_with_step() {
    let source = "for i = 1, 10, 2 sum = sum + i";
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[test]
fn test_parse_for_in() {
    let source = "for x in items print(x)";
    let result = Statement::parser().parse(source);
    result.unwrap();
}


#[test]
fn test_parse_function_def() {
    let source = r#"
function add(a, b)
    return a + b
end
"#;
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[test]
fn test_parse_local_function_def() {
    let source = "local function helper(x) return x * 2";
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[test]
fn test_parse_repeat_until() {
    let source = "repeat x = x + 1 until x > 10";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    matches!(result.unwrap(), Statement::RepeatLoop { .. });
}
