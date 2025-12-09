use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use chumsky::prelude::*;
use crate::lexer::{Token, LexError};

/// Format lexer errors using ariadne
pub fn format_lex_error(filename: &str, source: &str, error: &LexError) -> String {
    let (pos, message) = match error {
        LexError::UnexpectedChar { ch, pos } => {
            (*pos, format!("Unexpected character '{}'", ch))
        }
        LexError::UnterminatedString { pos } => {
            (*pos, "Unterminated string literal".to_string())
        }
        LexError::InvalidEscape { ch, pos } => {
            (*pos, format!("Invalid escape sequence '\\{}'", ch))
        }
        LexError::InvalidNumber { text, pos } => {
            (*pos, format!("Invalid number '{}'", text))
        }
    };

    let span = pos..pos + 1;
    let mut output = Vec::new();

    Report::build(ReportKind::Error, filename, span.start)
        .with_message("Lexical error")
        .with_label(
            Label::new((filename, span))
                .with_message(message)
                .with_color(Color::Red),
        )
        .finish()
        .write((filename, Source::from(source)), &mut output)
        .expect("Failed to write error report");

    String::from_utf8(output).expect("Invalid UTF-8 in error report")
}

/// Format parser errors using ariadne
pub fn format_parse_errors(filename: &str, source: &str, errors: &[Rich<Token>]) -> String {
    let mut output = Vec::new();

    for error in errors {
        let span = error.span().into_range();
        let expected = if error.expected().len() == 0 {
            "end of input".to_string()
        } else {
            error
                .expected()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let found = if let Some(found) = error.found() {
            format!("{:?}", found)
        } else {
            "end of input".to_string()
        };

        let message = format!("Expected {}, but found {}", expected, found);

        Report::build(ReportKind::Error, filename, span.start)
            .with_message("Parse error")
            .with_label(
                Label::new((filename, span))
                    .with_message(message)
                    .with_color(Color::Red),
            )
            .finish()
            .write((filename, Source::from(source)), &mut output)
            .expect("Failed to write error report");
    }

    String::from_utf8(output).expect("Invalid UTF-8 in error report")
}

/// Convenience function to parse a complete program and format any errors
pub fn parse_program(filename: &str, source: &str) -> Result<crate::ast::Spanned<crate::ast::Program>, String> {

    // Lex the source
    let (tokens, mut errs) = lexer().parse(src.as_str()).into_output_errors();

    // let tokens = match crate::lexer::lex(source) {
    //     Ok(tokens) => tokens,
    //     Err(error) => {
    //         return Err(format_lex_error(filename, source, &error));
    //     }
    // };

    // Parse the tokens - create input preserving original spans
    let end_span = tokens.last()
        .map(|t| t.span.end..t.span.end)
        .unwrap_or(0..0);

    let (result, errors) = crate::parser::program()
        .parse(tokens.as_slice().map(end_span.into(), |spanned| (&spanned.node, SimpleSpan::from(spanned.span.clone()))))
        .into_output_errors();

    if !errors.is_empty() {
        return Err(format_parse_errors(filename, source, &errors));
    }

    result.ok_or_else(|| "Unknown parse error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_parse_valid_program() {
        let source = indoc! {r#"
            pixelscript = {
                name = "test"
            }
            x = 42
        "#};

        let result = parse_program("test.lua", source);
        assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let source = indoc! {r#"
            pixelscript = {
                name = "test"
            }
            x =
        "#};

        let result = parse_program("test.lua", source);
        assert!(result.is_err());
        // The error message should contain useful information
        let error = result.unwrap_err();
        assert!(error.contains("Parse error") || error.contains("Expected"));
    }

    #[test]
    fn test_lex_error_formatting() {
        let source = "x = \"unterminated string";
        let result = parse_program("test.lua", source);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Unterminated string"));
    }
}
