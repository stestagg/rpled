use yansi::Condition;

/// Configuration options for AST formatting
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Number of spaces per indent level (0 = compact/inline mode)
    pub indent: usize,
    /// Color output control
    pub color: Condition,
}

impl FormatOptions {
    /// Create new formatting options with specified indentation
    pub fn new(indent: usize) -> Self {
        Self {
            indent,
            color: Condition::TTY_AND_COLOR,
        }
    }

    /// Create compact formatting options (no indentation, inline mode)
    pub fn compact() -> Self {
        Self::new(0)
    }

    /// Set color output control
    pub fn with_color(mut self, enabled: bool) -> Self {
        self.color = if enabled {
            Condition::ALWAYS
        } else {
            Condition::NEVER
        };
        self
    }

    /// Check if multiline mode is enabled
    pub fn is_multiline(&self) -> bool {
        self.indent > 0
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self::new(2)
    }
}
