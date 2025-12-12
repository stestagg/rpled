use crate::ast::expr::Expression;
use crate::ast::constant::Constant;
use crate::ast::NodeParser;
use chumsky::Parser;

#[test]
fn test_parse_literal_number() {
    let source = "42";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::Constant(Constant::Num(n)) = result.clone().unwrap() {
        assert_eq!(n, 42);
    } else {
        panic!("Expected literal number, got: {:?}", result);
    }
}

#[test]
fn test_parse_literal_float() {
    let source = "3.14";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::Constant(Constant::Float(f)) = result.unwrap() {
        assert_eq!(f, 3.14);
    } else {
        panic!("Expected literal float");
    }
}

#[test]
fn test_parse_literal_string() {
    let source = r#""hello world""#;
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::Constant(Constant::String(s)) = result.unwrap() {
        assert_eq!(s, "hello world");
    } else {
        panic!("Expected literal string");
    }
}

#[test]
fn test_parse_literal_bool_true() {
    let source = "true";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    matches!(result.unwrap(), Expression::Constant(Constant::True));
}

#[test]
fn test_parse_literal_bool_false() {
    let source = "false";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    matches!(result.unwrap(), Expression::Constant(Constant::False));
}

#[test]
fn test_parse_variable() {
    let source = "foo";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::Variable(name) = result.unwrap() {
        assert_eq!(name, "foo");
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_parse_qualified_name() {
    let source = "math.PI";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::Variable(name) = result.unwrap() {
        assert_eq!(name, "math.PI");
    } else {
        panic!("Expected qualified name variable");
    }
}

#[test]
fn test_parse_multi_level_qualified_name() {
    let source = "a.b.c";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::Variable(name) = result.unwrap() {
        assert_eq!(name, "a.b.c");
    } else {
        panic!("Expected qualified name variable");
    }
}

#[test]
fn test_parse_function_call() {
    let source = "foo(1, 2)";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::FunctionCall { name, args } = result.unwrap() {
        assert_eq!(name, "foo");
        assert_eq!(args.len(), 2);
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn test_parse_function_call_no_args() {
    let source = "foo()";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::FunctionCall { name, args } = result.unwrap() {
        assert_eq!(name, "foo");
        assert_eq!(args.len(), 0);
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn test_parse_qualified_function_call() {
    let source = "math.sin(x)";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::FunctionCall { name, args } = result.unwrap() {
        assert_eq!(name, "math.sin");
        assert_eq!(args.len(), 1);
    } else {
        panic!("Expected qualified function call");
    }
}

#[test]
fn test_parse_unary_minus() {
    let source = "-5";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::UnaryOp { op, .. } = result.unwrap() {
        assert_eq!(op, "-");
    } else {
        panic!("Expected unary minus");
    }
}

#[test]
fn test_parse_unary_not() {
    let source = "not true";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::UnaryOp { op, .. } = result.unwrap() {
        assert_eq!(op, "not");
    } else {
        panic!("Expected unary not");
    }
}

#[test]
fn test_parse_binop() {
    let source = "1 + 2";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::BinaryOp { left, op, right, .. } = result.unwrap() {
        assert_eq!(op, "+");
        assert_eq!(*left, Expression::Constant(Constant::Num(1)));
        assert_eq!(*right, Expression::Constant(Constant::Num(2)));
    } else {
        panic!("Expected binary op");
    }
}

#[test]
fn test_parse_binop_mult() {
    let source = "5 * 10";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::BinaryOp { left, op, right, .. } = result.unwrap() {
        assert_eq!(op, "*");
        assert_eq!(*left, Expression::Constant(Constant::Num(5)));
        assert_eq!(*right, Expression::Constant(Constant::Num(10)));
    } else {
        panic!("Expected binary op");
    }
}

#[test]
fn test_parse_table_empty() {
    let source = "{}";
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::TableDef(fields) = result.unwrap() {
        assert_eq!(fields.len(), 0);
    } else {
        panic!("Expected empty table");
    }
}

#[test]
fn test_parse_table_with_fields() {
    let source = r#"{name = "test", count = 5}"#;
    let result = Expression::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Expression::TableDef(fields) = result.unwrap() {
        assert_eq!(fields.len(), 2);
    } else {
        panic!("Expected table with fields");
    }
}
