use rstest::*;

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

    let Block { statements } = result.unwrap();
    assert_eq!(statements.len(), 2);
    if let Statement::Assignment { target, .. } = &statements[0] {
        assert_eq!(target, "x");
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

#[rstest]
#[case(r#"while sum < 100 do
    foo = 1
end"#)]
#[case("while x > 0 do x = x - 1 end")]
#[case("while true do; break; end")]
#[case(r#"while count < 10 do
    count = count + 1
    sum = sum + count
end"#)]
fn test_parse_while_loop(#[case] source: &str) {
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[test]
fn test_parse_if_statement() {
    let source = "if x > 5 then y = 1 end";
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    if let Statement::IfStmt { if_part: _, else_if_part, else_part } = result.unwrap() {
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
        assert_eq!(if_part.condition.compact_plain_format(), "BinaryOp: [Expression:[BinaryOp: [Expression:[Variable: i] % Expression:[Constant:15]]] == Expression:[Constant:0]]");
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

#[rstest]
#[case("for i = 1, 10 do; sum = sum + i; end")]
#[case("for i=1,10,2 do; j=i-1;sum = sum + j; end")]
#[case(r#"for i = 1, 10, 2 do
    sum = sum + i
end"#)]
#[case("for i = 1,10,2 do sum = sum + i end")]
#[case("for i=1,10,2 do j=i-1;sum = sum + j end")]
#[case("for i=1,10,2 do;; j=i-1;sum = sum + j; end")]
fn test_parse_for_numeric(#[case] source: &str) {
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[rstest]
#[case("for x in items do; print(x);end")]
#[case("for x in items do print(x) end")]
#[case(r#"for value in collection do
    sum = sum + value
end"#)]
fn test_parse_for_in(#[case] source: &str) {
    let result = Statement::parser().parse(source);
    result.unwrap();
}


#[rstest]
#[case(r#"function add(a, b)
    return a + b
end"#)]
#[case("function simple() return 42 end")]
#[case(r#"function multiply(x, y, z)
    local result = x * y * z
    return result
end"#)]
#[case(r#"function add(a, b)
    return a + b
end"#)]
#[case("function noargs() print(\"hello\") end")]
#[case("function noreturn(x) x = x + 1 end")]
fn test_parse_function_def(#[case] source: &str) {
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[rstest]
#[case("local function helper(x) return x * 2 end")]
#[case(r#"local function compute(a, b, c)
    return a + b + c
end"#)]
#[case("local function empty() end")]
#[case("local function single_param(n) return n * n end")]
fn test_parse_local_function_def(#[case] source: &str) {
    let result = Statement::parser().parse(source);
    result.unwrap();
}

#[rstest]
#[case("repeat x = x + 1 until x > 10")]
#[case(r#"repeat
    count = count + 1
    sum = sum + count
until count >= 100"#)]
#[case("repeat; i = i * 2; until i > 1000")]
#[case("repeat doSomething() until done")]
fn test_parse_repeat_until(#[case] source: &str) {
    let result = Statement::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    matches!(result.unwrap(), Statement::RepeatLoop { .. });
}

#[test]
fn test_parse_function_then_result() {
    let source = r#"function add(a, b)
    return a + b
end

result = add(5, 10)"#;

    let result = Block::parser().parse(source).into_result();
    if let Err(ref errs) = result {
        eprintln!("Parse errors: {:?}", errs);
    }
    assert!(result.is_ok(), "Should parse function followed by assignment");
}

#[test]
fn test_parse_program_with_function_then_result() {
    use crate::ast::program::Program;

    let source = r#"pixelscript = {
    name = "function_test"
}

function add(a, b)
    return a + b
end

result = add(5, 10)
"#;

    let result = Program::parser().parse(source).into_result();
    if let Err(ref errs) = result {
        eprintln!("Parse errors: {:?}", errs);
    }
    assert!(result.is_ok(), "Should parse program with function followed by assignment");
}

#[test]
fn test_parse_program_with_crlf() {
    use crate::ast::program::Program;

    // Using CRLF line endings like Windows files
    let source = "pixelscript = {\r\n    name = \"function_test\"\r\n}\r\n\r\nfunction add(a, b)\r\n    return a + b\r\nend\r\n\r\nresult = add(5, 10)\r\n";

    let result = Program::parser().parse(source).into_result();
    if let Err(ref errs) = result {
        eprintln!("Parse errors: {:?}", errs);
        for err in errs {
            eprintln!("  Error: {:?}", err);
        }
    }
    assert!(result.is_ok(), "Should parse program with CRLF line endings");
}
