use ariadne::{Label, Report, ReportKind, Source};
use chumsky::error::Rich;

/// Formats a list of chumsky parsing errors using ariadne for beautiful output
pub fn format_errors(
    source: &str,
    file_path: &str,
    errors: Vec<Rich<'_, char>>,
) -> String {
    let mut output = Vec::new();

    for e in errors {
        Report::build(ReportKind::Error, file_path, e.span().start)
            .with_message("Parse error")
            .with_note(e.contexts().map(|(c, s)| c.to_string()).collect::<Vec<_>>().join("\n"))
            .with_label(
                Label::new((file_path, e.span().into_range()))
                    .with_message(e.reason().to_string())
            )
            .finish()
            .write((file_path, Source::from(source)), &mut output)
            .expect("Failed to write error report");
    }

    String::from_utf8(output).expect("Failed to convert error output to string")
}
