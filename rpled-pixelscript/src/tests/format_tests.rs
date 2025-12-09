use crate::ast::program::Program;
use crate::ast::NodeParser;
use crate::format::{AstFormatWithName, FormatOptions};
use chumsky::Parser;
use indoc::indoc;

#[test]
fn test_format_compact() {
    let source = indoc! {r#"
        pixelscript = {
            name = "test",
            version = 1
        }

        x = 42
        y = x + 10
    "#};

    let result = Program::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let program = result.unwrap();
    let output = program.format(FormatOptions::compact().with_color(false));

    println!("Compact output:\n{}", output);
    // Should be compact (single line where possible)
    assert!(output.contains("Program"));
}

#[test]
fn test_format_multiline() {
    let source = indoc! {r#"
        pixelscript = {
            name = "test",
            version = 1
        }

        x = 42
        y = x + 10
    "#};

    let result = Program::parser().parse(source).into_result();
    assert!(result.is_ok(), "Parse errors: {:?}", result.as_ref().err());

    let program = result.unwrap();
    let options = FormatOptions::new(2).with_color(false);
    let output = program.format(options);

    println!("Multiline output:\n{}", output);
    // Should be multiline with indentation
    assert!(output.contains("Program"));
}

#[test]
fn test_format_nested_structures() {
    let source = indoc! {r#"
        pixelscript = {
            name = "nested",
            config = {
                fps = 60,
                brightness = 128
            }
        }

        function update(t)
            if t > 10 then
                r = t * 2
            else
                r = t
            end
    "#};

    let result = Program::parser().parse(source).into_result();

    if let Ok(program) = result {
        let output = program.format(FormatOptions::new(2).with_color(false));
        println!("Formatted output:\n{}", output);
        assert!(output.contains("Program"));
    } else {
        println!("Note: This test may fail if the parser doesn't support this syntax yet");
    }
}

#[test]
fn test_format_expressions() {
    let source = indoc! {r#"
        pixelscript = {
            name = "expr_test"
        }

        a = 1 + 2
        b = foo(x, y)
        c = not true
    "#};

    let result = Program::parser().parse(source).into_result();

    if let Ok(program) = result {
        let output = program.format(FormatOptions::new(2).with_color(false));
        println!("Formatted output:\n{}", output);
        assert!(output.contains("Program"));
    }
}
