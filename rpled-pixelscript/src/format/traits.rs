use yansi::Paint;
use super::{Formatter, FormatOptions};

/// Core trait for formatting the inner content of an AST node
///
/// Implementations output only the inner content, without the `NodeName:[...]` wrapper.
/// The wrapper is automatically added by the `AstFormatWithName` trait.
pub trait AstFormat {
    /// Format the inner content of this node
    fn format_into(&self, formatter: &mut Formatter);
    fn compact_format(&self) -> String {
        let mut formatter = Formatter::new(FormatOptions::compact());
        self.format_into(&mut formatter);
        formatter.into_string()
    }
    fn compact_plain_format(&self) -> String {
        let mut formatter = Formatter::new(FormatOptions::compact().with_color(false));
        self.format_into(&mut formatter);
        formatter.into_string()
    }
}

/// Wrapper trait that adds the `NodeName:[...]` wrapper to AST nodes
///
/// This trait automatically provides consistent formatting where each node
/// outputs as `NodeName:[inner_content]`. The `NODE_NAME` constant ensures
/// that the node name matches the type at compile time.
pub trait AstFormatWithName: AstFormat {
    /// The name of this AST node type (e.g., "Statement", "Expression", "Constant")
    const NODE_NAME: &'static str;
    const WRAP_CONTENT: bool = true;

    /// Format this node with its name wrapper: `NodeName:[inner_content]`
    fn format_with_name(&self, formatter: &mut Formatter) {
        formatter.write(Self::NODE_NAME.blue().bold());
        formatter.write(":".blue());

        if Self::WRAP_CONTENT {
            formatter.nested(|f| {
                self.format_into(f);
            });
        } else {
            formatter.nested_unwrapped(|f| {
                self.format_into(f);
            });
        }
    }

    /// Convenience method to format this node to a string
    fn format(&self, options: FormatOptions) -> String {
        let mut formatter = Formatter::new(options);
        self.format_with_name(&mut formatter);
        formatter.into_string()
    }
}
