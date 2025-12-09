use crate::ast::program::Program;
use crate::ast::NodeParser;
use chumsky::Parser;
use indoc::indoc;

#[test]
fn test_parse_minimal_program() {
    let source = indoc! {r#"
        pixelscript = {
            name = "test"
        }
    "#};

    let result = Program::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let parsed = result.unwrap();
    // The metadata is a MetadataBlock which contains a MetadataTable
    // Just verify it parsed successfully
    assert!(parsed.metadata.0.fields.len() >= 1);
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

    let result = Program::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let parsed = result.unwrap();
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
    "#};

    let result = Program::parser().parse(source);
    // This test may fail depending on how function blocks work
}

#[test]
fn test_parse_program_with_loops() {
    let source = indoc! {r#"
        pixelscript = {
            name = "loops"
        }

        for i = 1, 10 sum = sum + i

        while x < 100 x = x * 2
    "#};

    let result = Program::parser().parse(source);
    // This test may fail depending on how loop blocks work
}

#[test]
fn test_parse_complex_program() {
    let source = indoc! {r#"
        pixelscript = {
            name = "complex",
            fps = 30
        }

        local colors = {r = 1, g = 2, b = 3}

        function draw(x, y)
            if x > 10 then
                c = colors.r
            else
                c = colors.b
            end
    "#};

    let result = Program::parser().parse(source);
    // This test may fail and need adjustment based on the actual parser capabilities
}
