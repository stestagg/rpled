use yansi::{Paint, Painted};
use super::options::FormatOptions;

/// Formatter state machine for recursive AST formatting
pub struct Formatter {
    pub(crate) options: FormatOptions,
    nodes: Vec<OutputNode>,
    depth: usize,
} 

#[derive(Debug, Clone)]
enum OutputNode {
    /// Colored text fragment
    Text(Painted<String>),
    /// Nested content - rendered as [...] with optional newlines/indentation
    Nested(bool, Vec<OutputNode>),
    /// Separator - newline+indent (multiline) or space (compact)
    Separator,
}


impl Formatter {
    /// Create a new formatter with the given options
    pub fn new(options: FormatOptions) -> Self {
        Self {
            options,
            nodes: Vec::new(),
            depth: 0,
        }
    }

    /// Write colored text to the output
    pub fn write<T: std::fmt::Display>(&mut self, text: Painted<T>) {
        // Apply color condition and convert to string
        let s = text.whenever(self.options.color).to_string();
        // Store as Painted<String> (unpainted since color is already applied)
        self.nodes.push(OutputNode::Text(Painted::new(s)));
    }

    /// Write plain text to the output (no color)
    pub fn write_plain(&mut self, text: &str) {
        self.nodes.push(OutputNode::Text(Painted::new(text.to_string())));
    }

    /// Create a nested formatting context with brackets
    ///
    /// In multiline mode: adds newlines and indentation
    /// In compact mode: keeps everything inline
    pub fn nested<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        // Create child formatter with same options
        let mut child = Formatter {
            options: self.options.clone(),
            nodes: Vec::new(),
            depth: self.depth + 1,
        };

        // Let closure populate child
        f(&mut child);

        // Wrap child's nodes and append to parent
        self.nodes.push(OutputNode::Nested(true, child.nodes));
    }

    pub fn nested_unwrapped<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        // Create child formatter with same options
        let mut child = Formatter {
            options: self.options.clone(),
            nodes: Vec::new(),
            depth: self.depth + 1,
        };

        // Let closure populate child
        f(&mut child);

        // Wrap child's nodes and append to parent
        self.nodes.push(OutputNode::Nested(false, child.nodes));
    }

    /// Write a separator (newline in multiline, space in compact)
    pub fn separator(&mut self) {
        self.nodes.push(OutputNode::Separator);
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

    /// Get the final formatted output
    pub fn into_string(self) -> String {
        let mut output = String::new();
        self.fold_nodes(&self.nodes, &mut output, 0, false);
        output
    }

    /// Recursively fold OutputNode tree into formatted string
    ///
    /// `force_compact`: if true, render in compact mode regardless of options
    fn fold_nodes(&self, nodes: &[OutputNode], output: &mut String, depth: usize, force_compact: bool) {
        for node in nodes {
            match node {
                OutputNode::Text(painted) => {
                    output.push_str(&painted.to_string());
                }

                OutputNode::Nested(wrap_content, children) => {
                    // Colored opening bracket (green)
                    let mut end_len = 1;
                    if *wrap_content{
                        end_len = 0;
                        let open_bracket = "[".green().whenever(self.options.color);
                        output.push_str(&open_bracket.to_string());
                    }

                    if !force_compact && self.options.is_multiline() {
                        // Check if children would fit inline
                        let mut temp = String::new();
                        self.fold_nodes(children, &mut temp, depth, true);

                        let no_newlines = !temp.contains('\n');
                        let current_line_len = output.lines().last()
                            .map(|l| Self::visual_len(l))
                            .unwrap_or(0);
                        // +1 for the closing bracket we'll add
                        let would_fit = current_line_len + Self::visual_len(&temp) + end_len <= 80;

                        if no_newlines && would_fit && !children.is_empty() {
                            // Render inline
                            output.push_str(&temp);
                        } else {
                            // Multiline: newline, indent, children, newline, indent
                            if !children.is_empty() {
                                output.push('\n');
                                let child_depth = depth + 1;
                                self.write_indent(output, child_depth);

                                self.fold_nodes(children, output, child_depth, false);

                                output.push('\n');
                                self.write_indent(output, depth);
                            }
                        }
                    } else {
                        // Compact: inline children
                        self.fold_nodes(children, output, depth, force_compact);
                    }

                    // Colored closing bracket (green)
                    if *wrap_content{
                        let close_bracket = "]".green().whenever(self.options.color);
                        output.push_str(&close_bracket.to_string());
                    }
                }

                OutputNode::Separator => {
                    if !force_compact && self.options.is_multiline() {
                        output.push('\n');
                        self.write_indent(output, depth);
                    } else {
                        output.push(' ');
                    }
                }
            }
        }
    }

    /// Write indentation spaces based on depth
    fn write_indent(&self, output: &mut String, depth: usize) {
        let indent = " ".repeat(depth * self.options.indent);
        output.push_str(&indent);
    }

    /// Calculate the visual length of a string, excluding ANSI escape codes
    fn visual_len(s: &str) -> usize {
        let mut len = 0;
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Check if this is a CSI sequence (ESC[)
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    // Skip until we find the command byte (a letter)
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                len += 1;
            }
        }

        len
    }
}
