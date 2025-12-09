use rstest::*;
use std::path::PathBuf;
use crate::error::parse_program;
use crate::ast_format::{AstFormat, AstFormatOptions};
use pretty_assertions::assert_eq;

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

#[rstest]
fn test_fixtures(#[files("testprogs/*.pxs")] path: PathBuf) {
    // Disable colors in error output for consistent test comparisons
    std::env::set_var("NO_COLOR", "1");

    let fixture_data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {:?}: {}", path, e));

    let parsed_fixture = parse_fixture(&fixture_data);

    // Use a relative path for the filename to make tests portable across machines
    let filename = format!("testprogs/{}", path.file_name().unwrap().to_str().unwrap());

    // Try to parse the program
    let result = parse_program(
        &filename,
        &parsed_fixture.source
    );

    let actual_output = match result {
        Ok(program) => {
            // Successfully parsed - format the AST with 2-space inden
            let output = program.node.format(AstFormatOptions::new(2).with_color(false));
            output.to_string()
        }
        Err(err) => {
            // Parse or lex error - return the nicely formatted error message
            err
        }
    };

    assert_eq!(
        actual_output.trim(),
        parsed_fixture.expected_output.trim(),
        "Output did not match for fixture {:?}",
        path
    );
}
