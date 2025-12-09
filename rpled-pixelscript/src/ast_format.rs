use yansi::{Paint, Painted, Condition};

use crate::ast::*;
use std::fmt::{self, Display};


pub trait AstFormat: AstFormatInternal {
    fn format(&self, options: AstFormatOptions) -> FormattedAst {
        let mut formatted = FormattedAst::new(options);
        self.format_internal(&mut formatted);
        formatted
    }

}

impl<T: AstFormatInternal> AstFormat for T {}

pub struct AstFormatOptions {
    pub include_spans: bool,
    pub indent: usize,
    pub color: Option<bool>,
    condition: Condition,
}

impl Default for AstFormatOptions {
    fn default() -> Self {
        Self {
            include_spans: false,
            indent: 2,
            color: None,
            condition: Condition::TTY_AND_COLOR,
        }
    }
}

impl AstFormatOptions {
    pub fn new(indent: usize) -> Self {
        Self {
            include_spans: false,
            indent,
            color: None,
            condition: Condition::TTY_AND_COLOR,
        }
    }

    pub fn compact() -> Self {
        Self {
            include_spans: false,
            indent: 0,
            color: None,
            condition: Condition::TTY_AND_COLOR,
        }
    }

    pub fn with_spans(mut self) -> Self {
        self.include_spans = true;
        self
    }

    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    pub fn with_color(mut self, color: bool) -> Self {
        self.color = Some(color);
        self.condition = match color {
            true => Condition::ALWAYS,
            false => Condition::NEVER,
        };
        self
    }

    pub fn with_default_color(mut self) -> Self {
        self.color = None;
        self.condition = Condition::TTY_AND_COLOR;
        self
    }

    fn is_multiline(&self) -> bool {
        self.indent > 0
    }


}

pub struct FormattedAst {
    pub options: AstFormatOptions,

    result: Vec<String>,

    current_depth: usize,
    span_annotation: Option<String>,
    pending_nl: bool,
}

impl FormattedAst {
    fn new(options: AstFormatOptions) -> Self {
        Self {
            options,
            result: Vec::new(),
            current_depth: 0,
            span_annotation: None,
            pending_nl: false,
        }
    }

    fn line_prefix(&self) -> String {
        let indent = " ".repeat(self.current_depth * self.options.indent);
        return format!("\n{}", indent);
    }

    pub fn output<T: Display>(&mut self, text: Painted<T>)
    {
        if self.pending_nl {
            self.pending_nl = false;
            if self.options.is_multiline() {
                if let Some(annotation) = self.span_annotation.take() {
                    self.result.push("\t".to_string());
                    self.result.push(annotation);
                }
                self.result.push(self.line_prefix());
            } else {
                self.result.push(" ".to_string());
            }
        }
        self.result.push(text.whenever(self.options.condition).to_string());
    }

    pub fn start_block(&mut self) {
        self.current_depth += 1;
    }

    pub fn nl(&mut self) {
        self.pending_nl = true;
    }

    pub fn end_block(&mut self) {
        self.current_depth = self.current_depth.checked_sub(1).expect("Mismatched block ends");
    }

    pub fn item_end(&mut self) {
        self.nl();
    }

    pub fn output_item<T: Into<String>>(&mut self, item: T) {
        let text = item.into();
        match text.as_str() {
            "{" |"[" | "(" => {
                self.output(text.green());
                self.start_block();
                self.nl();
            },
            "]" | ")" | "}" => {
                self.end_block();
                self.nl();
                self.output(text.green());
            },
            "," | ";" => {
                self.output(text.cyan());
                self.item_end();
            },
            " " => {
                if self.options.is_multiline() {
                    self.nl();
                } else {
                    self.output(";".cyan());
                    self.nl();
                }
            }
            "=" => {
                self.output(Painted::new(format!(" {} ", text)).blue());
            },
            _ => todo!("Unhandled output item: {}", text),
        }
    }

    pub fn add_span_annotation(&mut self, annotation: String) {
        if self.options.is_multiline() {
            self.span_annotation = Some(annotation);
        }  else {
            self.output(Painted::new(annotation));
        }
    }


}

pub trait AstFormatInternal {
    fn format_internal(&self, output: &mut FormattedAst);
}

impl Display for FormattedAst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in &self.result {
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! ast_fmt {

    ($w:expr, $first:tt) => {
        ast_format!(@single $w, $first);
    };
    ($w:expr, $($item:tt),* $(,)?) => {{
        $(
            ast_format!(@single $w, $item);
        )*
    }};

    ($w:expr, ) => {};

    (@single $w:expr, NL) => {
        $w.nl();
    };

    (@single $w:expr, (?$($value:expr)+)) => {
        if let Some(v) = &$($value)+ {
            v.format_internal($w);
        }
    };

    (@single $w:expr, SP) => {
        $w.output(Painted::new(" "));
    };

    (@single $w:expr, $const:literal) => {
        $w.output_item($const);
    };

    (@single $w:expr, [each $sep:literal $vals:expr] ) => {
        for val in $vals {
            val.format_internal($w);
            $w.output_item($sep);
        }
    };

    (@single $w:expr, [$value:literal]) => {
        $w.output(Painted::new($value));
    };

    (@single $w:expr, [$value:literal $($sub:ident)+]) => {
        $w.output($value$(.$sub())*.whenever($w.options.condition));
    };

    (@single $w:expr, [$value:ident $($sub:ident)+]) => {
        $w.output($value$(.$sub())*.whenever($w.options.condition));
    };

    (@single $w:expr, ($($field:ident).+)) => {
        $($field).+.format_internal($w);
    };
}

// Impls

impl AstFormatInternal for String {
    fn format_internal(&self, output: &mut FormattedAst) {
        output.output(self.bold());
    }
}


impl<T: AstFormatInternal> AstFormatInternal for Spanned<T> {
    fn format_internal(&self, output: &mut FormattedAst) {
        if output.options.include_spans {
            let start_str = format!("{}", self.span.start);
            let end_str = format!("{}", self.span.end);
            let span_str = format!("[{}..{}]:", start_str.blue(), end_str.blue());
            output.add_span_annotation(span_str);
            self.node.format_internal(output);
        } else {
            self.node.format_internal(output);
        }
    }
}


impl AstFormatInternal for MetadataTable {
    fn format_internal(&self, output: &mut FormattedAst) {
        out!(output, '{', [each ',' &self.fields], '}');
    }
}


impl AstFormatInternal for MetadataField {
    fn format_internal(&self, output: &mut FormattedAst) {
        match self {
            MetadataField::Literal { name, value } => {
                out!(output, (name.node), '=', (value));
            }
            MetadataField::Table { name, table } => {
                out!(output, (name.node), '=', (table));
            }
            MetadataField::Call { name, args } => {
                out!(output, (name.node), '(', [each ',' args], ')');
            }
            MetadataField::List { name, items } => {
                out!(output, (name.node), '=', '{', [each ',' items], '}');
            }
        }
    }
}


impl AstFormatInternal for Literal {
    fn format_internal(&self, output: &mut FormattedAst) {
        match self {
            Literal::Number(n) => {
                output.output(n.yellow().to_owned());
            }
            Literal::Float(f) => {
                output.output(f.yellow().to_owned());
            }
            Literal::String(s) => {
                out!(output, ["\""], [s yellow], ["\""]);
                
            }
            Literal::Bool(b) => {
                output.output(Painted::new(format!("{}", b)));
            }
            Literal::Nil => {
                out!(output, ["nil"]);
            }
        }
    }
}


impl AstFormatInternal for Block {
    fn format_internal(&self, output: &mut FormattedAst) {
        out!(output, '{', 
            [each ' ' &self.statements],
            (?self.return_stmt),
        '}'
        );
    }
}

impl AstFormatInternal for Statement {
    fn format_internal(&self, output: &mut FormattedAst) {
        match self {
            Statement::Assignment(a) => {
                out!(output, (a.var.node), '=', (a.value));
            }
            Statement::FunctionCall(fc) => fc.format_internal(output),
            Statement::DoBlock(block) => {
                out!(output, "do");
                output.current_depth += 1;
                block.format_internal(output);
                output.current_depth -= 1;
                out!(output, ["end" magenta]);
            }
            Statement::While(w) => {
                out!(output, ["while " magenta], (w.condition), SP, (w.block));
            }
            Statement::Repeat(r) => {
                out!(output, ["repeat " magenta], (r.block), [" until " magenta], (r.condition));
            }
            Statement::If(i) => {
                out!(output, ["if " magenta], (i.condition), [" then " magenta]);
                i.then_block.format_internal(output);
                for (cond, block) in &i.elseif_branches {
                    out!(output, [" elseif " magenta], (cond), [" then " magenta]);
                    block.format_internal(output);
                }
                if let Some(else_block) = &i.else_block {
                    out!(output, [" else " magenta]);
                    else_block.format_internal(output);
                }
                out!(output, [" end"]);
            }
            Statement::ForNum(f) => {
                out!(output, ["for"], SP, (f.var.node), '=', (f.start), [", "], (f.end));
                if let Some(step) = &f.step {
                    out!(output, [", "], (step));
                }
                out!(output, SP, ["do " magenta]);
                f.block.format_internal(output);
                out!(output, SP, ["end" magenta]);
            }
            Statement::ForIn(f) => {
                out!(output, ["for " magenta], (f.var.node), [" in " magenta], (f.iterator.node), [" do" magenta]);
                f.block.format_internal(output);
                out!(output, SP, ["end" magenta]);
            }
            Statement::FunctionDef(f) => {
                out!(output, ["function " magenta], (f.name.node), '(');
                for (i, param) in f.params.iter().enumerate() {
                    param.format_internal(output);
                    if i < f.params.len() - 1 {
                        out!(output, ',');
                    }
                }
                out!(output, ')');
                f.block.format_internal(output);
            }
            Statement::LocalFunctionDef(f) => {
                out!(output, ["local function " magenta], (f.name.node), '(');
                for (i, param) in f.params.iter().enumerate() {
                    param.format_internal(output);
                    if i < f.params.len() - 1 {
                        out!(output, ',');
                    }
                }
                out!(output, ')');
                f.block.format_internal(output);
            }
            Statement::LocalDecl(d) => {
                out!(output, ["local" magenta], SP, (d.name.node));
                if let Some(value) = &d.value {
                    out!(output, '=', (value));
                }
            }
            Statement::Break => {
                out!(output, ["break" magenta]);
            }
        }
    }
}

impl AstFormatInternal for Expr {
    fn format_internal(&self, output: &mut FormattedAst) {
        match self {
            Expr::Literal(lit) => lit.format_internal(output),
            Expr::Var(name) => {
                output.output(Painted::new(name.clone()));
            }
            Expr::QualifiedName(parts) => {
                output.output(Painted::new(parts.join(".")));
            }
            Expr::Index(idx) => {
                out!(output, (idx.base), (idx));
            }
            Expr::Parenthesized(e) => {
                out!(output, '(', (e), ')');                
            }
            Expr::BinaryOp(b) => {
                out!(output, (b.left), SP, (b.op.node), SP, (b.right));
            }
            Expr::UnaryOp(u) => {
                out!(output, (u.op.node), (u.operand));
            }
            Expr::TableConstructor(tc) => tc.format_internal(output),
            Expr::FunctionCall(fc) => fc.format_internal(output),
        }
    }
}

impl AstFormatInternal for IndexExpr {
    fn format_internal(&self, output: &mut FormattedAst) {
        out!(output, (self.base), ["["], (self.index), ["]"]);
    }
}

impl AstFormatInternal for PrefixExpr {
    fn format_internal(&self, output: &mut FormattedAst) {
        match self {
            PrefixExpr::Var(name) => {
                output.output(Painted::new(name.clone()));
            }
            PrefixExpr::QualifiedName(parts) => {
                output.output(Painted::new(parts.join(".")));
            }
        }
    }
}

impl AstFormatInternal for FunctionCall {
    fn format_internal(&self, output: &mut FormattedAst) {
        out!(output, (self.func), '(');
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                out!(output, ',');
            }
            arg.format_internal(output);
        }
        out!(output, ')');
    }
}

impl AstFormatInternal for TableConstructor {
    fn format_internal(&self, output: &mut FormattedAst) {
        out!(output, '{');
        for (i, field) in self.fields.iter().enumerate() {
            field.format_internal(output);
            if i < self.fields.len() - 1 {
                out!(output, ',');
            }
        }
        out!(output, '}');
    }
}


impl AstFormatInternal for TableField {
    fn format_internal(&self, output: &mut FormattedAst) {
        match self {
            TableField::Indexed { key, value } => {
                out!(output, '[', (key), ']', '=', (value));
            }
            TableField::Named { name, value } => {
                out!(output, (name.node), '=', (value));
            }
            TableField::Value(val) => {
                val.format_internal(output);
            }
        }
    }
}

impl AstFormatInternal for ReturnStmt {
    fn format_internal(&self, output: &mut FormattedAst) {
        out!(output, ["return" magenta], SP, (?self.value));
    }
}


impl AstFormatInternal for BinOp {
    fn format_internal(&self, output: &mut FormattedAst) {
        output.output(Painted::new(self.clone()));
    }
}

impl AstFormatInternal for UnOp {
    fn format_internal(&self, output: &mut FormattedAst) {
        output.output(Painted::new(self.clone()));
    }
}