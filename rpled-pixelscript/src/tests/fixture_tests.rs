use crate::ast::program::Program;
use crate::ast::NodeParser;
use crate::format::{AstFormatWithName, FormatOptions};
use chumsky::Parser;
use pretty_assertions::assert_eq;
use std::path::PathBuf;

const OUTPUT_SEPARATOR: &str = "=== OUTPUT ===";

struct ParsedFixture {
    source: String,
    expected_output: String,
}

fn parse_fixture(data: &str) -> ParsedFixture {
    let (source_section, output_section) = data
        .rsplit_once(OUTPUT_SEPARATOR)
        .expect("Fixture must contain '=== OUTPUT ===' separator");

    ParsedFixture {
        source: source_section.trim().to_string(),
        // Normalize line endings to LF for consistent comparison
        expected_output: output_section.trim().replace("\r\n", "\n"),
    }
}

// Helper function to test a single fixture file
fn test_fixture_file(path: PathBuf) {
    // Disable colors in error output for consistent test comparisons
    unsafe {
        std::env::set_var("NO_COLOR", "1");
    }

    let fixture_data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {:?}: {}", path, e));

    let parsed_fixture = parse_fixture(&fixture_data);

    // Try to parse the program
    let result = Program::parser().parse(&parsed_fixture.source).into_result();

    let actual_output = match result {
        Ok(program) => {
            // Successfully parsed - format the AST with 2-space indent
            program.format(FormatOptions::new(2).with_color(false))
        }
        Err(errs) => {
            // Parse error - return the error message
            format!("Parse errors: {:?}", errs)
        }
    };

    assert_eq!(
        actual_output.trim(),
        parsed_fixture.expected_output.trim(),
        "Output did not match for fixture {:?}",
        path
    );
}

#[test]
fn test_fixture_simple() {
    let path = PathBuf::from("testprogs/simple.pxs");
    if path.exists() {
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_simple: file not found");
    }
}

#[test]
fn test_fixture_function() {
    let path = PathBuf::from("testprogs/function.pxs");
    if path.exists() {
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_function: file not found");
    }
}

#[test]
fn test_fixture_loops() {
    let path = PathBuf::from("testprogs/loops.pxs");
    if path.exists() {
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_loops: file not found");
    }
}

#[test]
fn test_fixture_fizzbuzz() {
    let path = PathBuf::from("testprogs/fizzbuzz.pxs");
    if path.exists() {
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_fizzbuzz: file not found");
    }
}

#[test]
fn test_fixture_comprehensive() {
    let path = PathBuf::from("testprogs/comprehensive.pxs");
    if path.exists() {
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_comprehensive: file not found");
    }
}

#[test]
fn test_fixture_nested_metadata() {
    let path = PathBuf::from("testprogs/nested_metadata.pxs");
    if path.exists() {
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_nested_metadata: file not found");
    }
}

// Error test fixtures
#[test]
fn test_fixture_lex_error() {
    let path = PathBuf::from("testprogs/lex_error.pxs");
    if path.exists() {
        // This should produce an error
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_lex_error: file not found");
    }
}

#[test]
fn test_fixture_syntax_error() {
    let path = PathBuf::from("testprogs/syntax_error.pxs");
    if path.exists() {
        // This should produce an error
        test_fixture_file(path);
    } else {
        println!("Skipping test_fixture_syntax_error: file not found");
    }
}
