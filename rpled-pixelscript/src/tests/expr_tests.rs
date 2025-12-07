use crate::lexer::lex;
use crate::parsers::expr::expr;
use crate::ast::{Expr, BinOp, UnOp, Literal};
use chumsky::Parser;

#[test]
fn test_parse_literal_expr() {
    let source = "42";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    if let Expr::Literal(Literal::Number(n)) = result.unwrap().node {
        assert_eq!(n, 42);
    } else {
        panic!("Expected literal number");
    }
}

#[test]
fn test_parse_variable() {
    let source = "foo";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::Var(name) = result.unwrap().node {
        assert_eq!(name, "foo");
    } else {
        panic!("Expected variable");
    }
}

#[test]
fn test_parse_binary_add() {
    let source = "1 + 2";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::BinaryOp(binop) = result.unwrap().node {
        assert_eq!(binop.op.node, BinOp::Add);
    } else {
        panic!("Expected binary operation");
    }
}

#[test]
fn test_operator_precedence_mul_before_add() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let source = "1 + 2 * 3";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let parsed = result.unwrap();

    // Should be Add at top level
    if let Expr::BinaryOp(add_op) = parsed.node {
        assert_eq!(add_op.op.node, BinOp::Add);

        // Left should be 1
        if let Expr::Literal(Literal::Number(n)) = add_op.left.node {
            assert_eq!(n, 1);
        } else {
            panic!("Expected left to be 1");
        }

        // Right should be (2 * 3)
        if let Expr::BinaryOp(mul_op) = add_op.right.node {
            assert_eq!(mul_op.op.node, BinOp::Mul);
        } else {
            panic!("Expected right to be multiplication");
        }
    } else {
        panic!("Expected addition at top level");
    }
}

#[test]
fn test_operator_precedence_parens() {
    // (1 + 2) * 3 should parse with addition first
    let source = "(1 + 2) * 3";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let parsed = result.unwrap();

    // Should be Mul at top level
    if let Expr::BinaryOp(mul_op) = parsed.node {
        assert_eq!(mul_op.op.node, BinOp::Mul);

        // Left should be parenthesized (1 + 2)
        if let Expr::Parenthesized(inner) = mul_op.left.node {
            if let Expr::BinaryOp(add_op) = inner.node {
                assert_eq!(add_op.op.node, BinOp::Add);
            } else {
                panic!("Expected addition inside parens");
            }
        } else {
            panic!("Expected parenthesized expression");
        }
    } else {
        panic!("Expected multiplication at top level");
    }
}

#[test]
fn test_right_associative_power() {
    // 2 ^ 3 ^ 4 should parse as 2 ^ (3 ^ 4)
    let source = "2 ^ 3 ^ 4";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    let parsed = result.unwrap();

    // Should be Pow at top level
    if let Expr::BinaryOp(pow_op) = parsed.node {
        assert_eq!(pow_op.op.node, BinOp::Pow);

        // Left should be 2
        if let Expr::Literal(Literal::Number(n)) = pow_op.left.node {
            assert_eq!(n, 2);
        } else {
            panic!("Expected left to be 2");
        }

        // Right should be (3 ^ 4)
        if let Expr::BinaryOp(inner_pow) = pow_op.right.node {
            assert_eq!(inner_pow.op.node, BinOp::Pow);
        } else {
            panic!("Expected right to be another power operation");
        }
    } else {
        panic!("Expected power at top level");
    }
}

#[test]
fn test_unary_minus() {
    let source = "-5";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::UnaryOp(unop) = result.unwrap().node {
        assert_eq!(unop.op.node, UnOp::Neg);
    } else {
        panic!("Expected unary operation");
    }
}

#[test]
fn test_unary_not() {
    let source = "not true";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::UnaryOp(unop) = result.unwrap().node {
        assert_eq!(unop.op.node, UnOp::Not);
    } else {
        panic!("Expected unary not operation");
    }
}

#[test]
fn test_comparison_operators() {
    let test_cases = vec![
        ("1 < 2", BinOp::Lt),
        ("1 <= 2", BinOp::Le),
        ("1 > 2", BinOp::Gt),
        ("1 >= 2", BinOp::Ge),
        ("1 == 2", BinOp::Eq),
        ("1 ~= 2", BinOp::Ne),
    ];

    for (source, expected_op) in test_cases {
        let tokens = lex(source).unwrap();
        let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

        let (result, errors) = expr().parse(&token_slice).into_output_errors();
        assert!(errors.is_empty(), "Parse errors for '{}': {:?}", source, errors);

        if let Expr::BinaryOp(binop) = result.unwrap().node {
            assert_eq!(binop.op.node, expected_op, "For expression '{}'", source);
        } else {
            panic!("Expected binary operation for '{}'", source);
        }
    }
}

#[test]
fn test_logical_operators() {
    let source = "true and false or true";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    // Should parse as (true and false) or true
    // Or has lower precedence than And
    if let Expr::BinaryOp(or_op) = result.unwrap().node {
        assert_eq!(or_op.op.node, BinOp::Or);

        // Left should be And
        if let Expr::BinaryOp(and_op) = or_op.left.node {
            assert_eq!(and_op.op.node, BinOp::And);
        } else {
            panic!("Expected and on left");
        }
    } else {
        panic!("Expected or at top level");
    }
}

#[test]
fn test_function_call() {
    let source = "foo(1, 2)";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::FunctionCall(call) = result.unwrap().node {
        assert_eq!(call.args.len(), 2);
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn test_index_expression() {
    let source = "arr[5]";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::Index(_) = result.unwrap().node {
        // Success
    } else {
        panic!("Expected index expression");
    }
}

#[test]
fn test_table_constructor_empty() {
    let source = "{}";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::TableConstructor(tc) = result.unwrap().node {
        assert_eq!(tc.fields.len(), 0);
    } else {
        panic!("Expected table constructor");
    }
}

#[test]
fn test_table_constructor_values() {
    let source = r#"{1, 2, "three"}"#;
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::TableConstructor(tc) = result.unwrap().node {
        assert_eq!(tc.fields.len(), 3);
    } else {
        panic!("Expected table constructor");
    }
}

#[test]
fn test_table_constructor_named_fields() {
    let source = r#"{name = "test", count = 5}"#;
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Expr::TableConstructor(tc) = result.unwrap().node {
        assert_eq!(tc.fields.len(), 2);
    } else {
        panic!("Expected table constructor");
    }
}

#[test]
fn test_complex_expression() {
    let source = "x + y * 2 - foo(a, b) / 3";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = expr().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());
}
