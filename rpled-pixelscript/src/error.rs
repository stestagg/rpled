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
        let mut builder = Report::build(ReportKind::Error, file_path, e.span().start)
            .with_message("Parse error")
            .with_label(
                Label::new((file_path, e.span().into_range()))
                    .with_message(e.reason().to_string())
            );
            let context = e.contexts().map(|(c, _s)| c.to_string()).collect::<Vec<_>>().join("\n");
            if !context.is_empty() {
                builder = builder.with_note(context);
            }
            builder
            .finish()
            .write((file_path, Source::from(source)), &mut output)
            .expect("Failed to write error report");
    }

    String::from_utf8(output).expect("Failed to convert error output to string")
}
