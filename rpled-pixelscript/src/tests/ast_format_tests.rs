use crate::ast_format::{AstFormat, AstFormatOptions};
use crate::error::parse_program;
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

    let program = parse_program("test.lua", source).unwrap();
    let output = program.node.format(AstFormatOptions::compact()).to_string();
    // Should be compact (single line where possible)
    assert!(output.contains("Block { statements: [x = 42, y = x + 10] }"));
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

    let program = parse_program("test.lua", source).unwrap();
    
    let options = AstFormatOptions::new(2).with_color(false); // 2-space indent
    let output = program.node.format(options).to_string();

    // Should be multiline with indentation
    assert!(output.contains("Program {\n"));
    assert!(output.contains("  metadata:"));
    assert!(output.contains("  block:"));
    assert!(output.contains("    statements: [\n"));
    assert!(output.contains("      x = 42"));
    assert!(output.contains("      y = x + 10"));
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
                return t * 2
            else
                return t
            end
        end
    "#};

    let program = parse_program("test.lua", source).unwrap();

    let output = program.node.format(AstFormatOptions::new(2)).to_string();

    println!("Formatted output:\n{}", output);

    // Should have nested indentation
    assert!(output.contains("config = {\n"));
    assert!(output.contains("      fps = 60")); // Inside nested config table
    assert!(output.contains("if t > 10 then\n"));
    assert!(output.contains("return t * 2")); // Check for content without specific spacing
}

#[test]
fn test_format_with_spans() {
    let source = "pixelscript = { name = \"test\" }\nx = 1";

    let program = parse_program("test.lua", source).unwrap();

    let mut output = String::new();
    let mut options = AstFormatOptions::compact();
    options.include_spans = true;
    let output = program.node.format(options).to_string();

    // Should include span information
    assert!(output.contains("["));
    assert!(output.contains(".."));
    assert!(output.contains("]:"));
}
