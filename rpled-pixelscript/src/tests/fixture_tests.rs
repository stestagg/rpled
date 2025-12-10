use crate::ast::program::Program;
use crate::ast::NodeParser;
use crate::error::format_errors;
use crate::format::{AstFormatWithName, FormatOptions};
use chumsky::Parser;
use pretty_assertions::assert_eq;
use rstest::*;
use std::path::PathBuf;

#[rstest]
fn test_fixture(#[files("../testprogs/*/script.pxl")] script_path: PathBuf) {
    // Disable colors in error output for consistent test comparisons
    unsafe {
        std::env::set_var("NO_COLOR", "1");
    }

    // Derive the ast.txt path from the script.pxl path
    let ast_path = script_path.with_file_name("ast.txt");

    let source = std::fs::read_to_string(&script_path)
        .unwrap_or_else(|e| panic!("Failed to read script {:?}: {}", script_path, e));

    let expected_output = std::fs::read_to_string(&ast_path)
        .unwrap_or_else(|e| panic!("Failed to read expected output {:?}: {}", ast_path, e));

    // Try to parse the program
    let result = Program::parser().parse(&source).into_result();

    let actual_output = match result {
        Ok(program) => {
            // Successfully parsed - format the AST with 2-space indent
            program.format(FormatOptions::new(2).with_color(false))
        }
        Err(errs) => {
            // Parse error - format using ariadne
            // Use a simplified file path for display (just the testprogs/... part)
            let display_path = script_path
                .to_string_lossy()
                .replace("\\", "/")
                .split("/")
                .skip_while(|s| *s != "testprogs")
                .collect::<Vec<_>>()
                .join("/");

            format_errors(&source, &display_path, errs)
        }
    };

    // Normalize line endings for comparison
    let expected_normalized = expected_output.trim().replace("\r\n", "\n");
    let actual_normalized = actual_output.trim().replace("\r\n", "\n");

    assert_eq!(
        actual_normalized,
        expected_normalized,
        "Output did not match for fixture {:?}",
        script_path
    );
}
