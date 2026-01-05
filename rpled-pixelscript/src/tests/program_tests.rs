use crate::lexer::lex;
use crate::parser::program;
use crate::ast::Program;
use chumsky::Parser;
use indoc::indoc;

#[test]
fn test_parse_minimal_program() {
    let source = indoc! {r#"
        pixelscript = {
            name = "test"
        }
    "#};

    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = program().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    let parsed: Program = result.unwrap().node;
    assert_eq!(parsed.metadata.node.name.node, "pixelscript");
    assert_eq!(parsed.block.statements.len(), 0);
}

#[test]
fn test_parse_program_with_code() {
    let source = indoc! {r#"
        pixelscript = {
            name = "test",
            version = 1
        }

        x = 10
        y = 20
    "#};

    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = program().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    let parsed: Program = result.unwrap().node;
    assert_eq!(parsed.metadata.node.table.node.fields.len(), 2);
    assert_eq!(parsed.block.statements.len(), 2);
}

#[test]
fn test_parse_program_with_function() {
    let source = indoc! {r#"
        pixelscript = {
            name = "animation"
        }

        function update(t)
            local x = t * 10
            return x
        end
    "#};

    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = program().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    let parsed: Program = result.unwrap().node;
    assert_eq!(parsed.block.statements.len(), 1);
}

#[test]
fn test_parse_program_with_loops() {
    let source = indoc! {r#"
        pixelscript = {
            name = "loops"
        }

        for i = 1, 10 do
            sum = sum + i
        end

        while x < 100 do
            x = x * 2
        end
    "#};

    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = program().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    let parsed: Program = result.unwrap().node;
    assert_eq!(parsed.block.statements.len(), 2);
}

#[test]
fn test_parse_program_with_return() {
    let source = indoc! {r#"
        pixelscript = {
            name = "test"
        }

        x = 5
        return x + 1
    "#};

    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = program().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    let parsed: Program = result.unwrap().node;
    assert_eq!(parsed.block.statements.len(), 1);
    assert!(parsed.block.return_stmt.is_some());
}

#[test]
fn test_parse_complex_program() {
    let source = indoc! {r#"
        pixelscript = {
            name = "complex",
            fps = 30,
            resolution = (800, 600)
        }

        local colors = {1, 2, 3}

        function draw(x, y)
            if x > 10 then
                return colors[1]
            elseif x > 5 then
                return colors[2]
            else
                return colors[3]
            end
        end

        for i = 1, 10 do
            result = draw(i, i)
        end

        return result
    "#};

    let tokens = lex(source).unwrap();
    let token_slice: Vec<_> = tokens.iter().map(|t| t.node.clone()).collect();

    let (result, errors) = program().parse(&token_slice).into_output_errors();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
    assert!(result.is_some());

    let parsed: Program = result.unwrap().node;
    assert_eq!(parsed.metadata.node.table.node.fields.len(), 3);
    assert_eq!(parsed.block.statements.len(), 3);
    assert!(parsed.block.return_stmt.is_some());
}
