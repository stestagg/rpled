#[test]
fn test_parse_function_then_result() {
    use crate::ast::{Block, NodeParser};
    use chumsky::Parser;

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
