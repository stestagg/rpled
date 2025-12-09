use yansi::{Paint, Painted};
use super::options::FormatOptions;

/// Formatter state machine for recursive AST formatting
pub struct Formatter {
    pub(crate) options: FormatOptions,
    output: String,
    depth: usize,
}

impl Formatter {
    /// Create a new formatter with the given options
    pub fn new(options: FormatOptions) -> Self {
        Self {
            options,
            output: String::new(),
            depth: 0,
        }
    }

    /// Write colored text to the output
    pub fn write<T: std::fmt::Display>(&mut self, text: Painted<T>) {
        self.output
            .push_str(&text.whenever(self.options.color).to_string());
    }

    /// Write plain text to the output (no color)
    pub fn write_plain(&mut self, text: &str) {
        self.output.push_str(text);
    }

    /// Create a nested formatting context with brackets
    ///
    /// In multiline mode: adds newlines and indentation
    /// In compact mode: keeps everything inline
    pub fn nested<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        if self.options.is_multiline() {
            // Multiline mode: brackets with indentation
            self.write("[".green());
            self.depth += 1;
            self.write_newline();

            f(self);

            self.depth -= 1;
            self.write_newline();
            self.write("]".green());
        } else {
            // Compact mode: inline brackets
            self.write("[".green());
            f(self);
            self.write("]".green());
        }
    }

    /// Write a separator (newline in multiline, space in compact)
    pub fn separator(&mut self) {
        if self.options.is_multiline() {
            self.write_newline();
        } else {
            self.write_plain(" ");
        }
    }

    /// Write a comma followed by a separator
    pub fn comma(&mut self) {
        self.write(",".cyan());
        self.separator();
    }

    /// Write a tagged value: `tag: value`
    /// Tag is colored cyan, colon is plain, value is colored yellow
    pub fn write_tagged<T: std::fmt::Display>(&mut self, tag: &str, value: T) {
        self.write(tag.cyan());
        self.write_plain(": ");
        self.write(value.yellow());
    }

    /// Format a labeled field: `label: value`
    pub fn field<F>(&mut self, label: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.write(label.cyan());
        self.write(": ".cyan());
        f(self);
    }

    /// Format an optional field (only outputs if Some)
    pub fn optional<T, F>(&mut self, label: &str, value: &Option<T>, f: F)
    where
        F: FnOnce(&mut Self, &T),
    {
        if let Some(v) = value {
            self.field(label, |fmt| f(fmt, v));
        }
    }

    /// Format a list of items with separators
    ///
    /// Uses commas to separate items
    pub fn list<T, F>(&mut self, items: &[T], f: F)
    where
        F: Fn(&mut Self, &T),
    {
        for (i, item) in items.iter().enumerate() {
            f(self, item);
            if i < items.len() - 1 {
                self.comma();
            }
        }
    }

    /// Write a newline with proper indentation
    fn write_newline(&mut self) {
        if self.options.is_multiline() {
            self.output.push('\n');
            let indent = "  ".repeat(self.depth * self.options.indent);
            self.output.push_str(&indent);
        }
    }

    /// Get the final formatted output
    pub fn into_string(self) -> String {
        self.output
    }
}
