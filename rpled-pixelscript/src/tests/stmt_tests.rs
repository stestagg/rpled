use crate::lexer::lex;
use crate::parsers::stmt::statement;
use crate::ast::{Statement, Assignment, WhileStmt, IfStmt, ForNumStmt, ForInStmt, FunctionDef, LocalDecl};
use chumsky::Parser;

#[test]
fn test_parse_assignment() {
    let source = "x = 42";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    if let Statement::Assignment(Assignment { var, .. }) = result.unwrap().node {
        assert_eq!(var.node, "x");
    } else {
        panic!("Expected assignment");
    }
}

#[test]
fn test_parse_function_call() {
    let source = "foo(1, 2)";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::FunctionCall(call) = result.unwrap().node {
        assert_eq!(call.args.len(), 2);
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn test_parse_while_loop() {
    let source = "while x < 10 do x = x + 1 end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::While(WhileStmt { block, .. }) = result.unwrap().node {
        assert_eq!(block.statements.len(), 1);
    } else {
        panic!("Expected while statement");
    }
}

#[test]
fn test_parse_if_statement() {
    let source = "if x > 5 then y = 1 end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::If(IfStmt { then_block, .. }) = result.unwrap().node {
        assert_eq!(then_block.statements.len(), 1);
    } else {
        panic!("Expected if statement");
    }
}

#[test]
fn test_parse_if_else_statement() {
    let source = "if x > 5 then y = 1 else y = 0 end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::If(IfStmt { then_block, else_block, .. }) = result.unwrap().node {
        assert_eq!(then_block.statements.len(), 1);
        assert!(else_block.is_some());
        assert_eq!(else_block.unwrap().statements.len(), 1);
    } else {
        panic!("Expected if-else statement");
    }
}

#[test]
fn test_parse_if_elseif_statement() {
    let source = "if x > 10 then y = 2 elseif x > 5 then y = 1 else y = 0 end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::If(IfStmt { elseif_branches, .. }) = result.unwrap().node {
        assert_eq!(elseif_branches.len(), 1);
    } else {
        panic!("Expected if-elseif statement");
    }
}

#[test]
fn test_parse_for_numeric() {
    let source = "for i = 1, 10 do sum = sum + i end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::ForNum(ForNumStmt { var, step, .. }) = result.unwrap().node {
        assert_eq!(var.node, "i");
        assert!(step.is_none());
    } else {
        panic!("Expected numeric for statement");
    }
}

#[test]
fn test_parse_for_numeric_with_step() {
    let source = "for i = 1, 10, 2 do sum = sum + i end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::ForNum(ForNumStmt { step, .. }) = result.unwrap().node {
        assert!(step.is_some());
    } else {
        panic!("Expected numeric for statement with step");
    }
}

#[test]
fn test_parse_for_in() {
    let source = "for x in items do print(x) end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::ForIn(ForInStmt { var, iterator, .. }) = result.unwrap().node {
        assert_eq!(var.node, "x");
        assert_eq!(iterator.node, "items");
    } else {
        panic!("Expected for-in statement");
    }
}

#[test]
fn test_parse_function_def() {
    let source = "function add(a, b) return a + b end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::FunctionDef(FunctionDef { name, params, block }) = result.unwrap().node {
        assert_eq!(name.node, "add");
        assert_eq!(params.len(), 2);
        assert!(block.return_stmt.is_some());
    } else {
        panic!("Expected function definition");
    }
}

#[test]
fn test_parse_local_function_def() {
    let source = "local function helper(x) return x * 2 end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    matches!(result.unwrap().node, Statement::LocalFunctionDef(_));
}

#[test]
fn test_parse_local_decl_with_value() {
    let source = "local x = 10";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::LocalDecl(LocalDecl { name, value }) = result.unwrap().node {
        assert_eq!(name.node, "x");
        assert!(value.is_some());
    } else {
        panic!("Expected local declaration");
    }
}

#[test]
fn test_parse_local_decl_without_value() {
    let source = "local x";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::LocalDecl(LocalDecl { value, .. }) = result.unwrap().node {
        assert!(value.is_none());
    } else {
        panic!("Expected local declaration without value");
    }
}

#[test]
fn test_parse_break() {
    let source = "break";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    matches!(result.unwrap().node, Statement::Break);
}

#[test]
fn test_parse_do_block() {
    let source = "do x = 5 y = 10 end";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    if let Statement::DoBlock(block) = result.unwrap().node {
        assert_eq!(block.statements.len(), 2);
    } else {
        panic!("Expected do block");
    }
}

#[test]
fn test_parse_repeat_until() {
    let source = "repeat x = x + 1 until x > 10";
    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = statement().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);

    matches!(result.unwrap().node, Statement::Repeat(_));
}
