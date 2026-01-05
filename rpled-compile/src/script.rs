use anyhow::{anyhow, bail, Result};
use std::borrow::Cow;
use luaparse::ast::*;
use luaparse::token::TokenValue;
use luaparse::{AstDescend, VisitorMut};

use crate::parser::ParsedLua;

// Helper function to extract string from TokenValue
fn token_to_str<'a>(token_value: &'a TokenValue<'a>) -> Cow<'a, str> {
    match token_value {
        TokenValue::Ident(s) => Cow::Borrowed(s),
        TokenValue::String { value, .. } => {
            match std::str::from_utf8(value) {
                Ok(s) => Cow::Owned(s.to_string()),
                Err(_) => Cow::Borrowed(""),
            }
        }
        _ => Cow::Borrowed(""),
    }
}

#[derive(Debug)]
pub struct PixelscriptHeader {
    pub name: String,
    pub modules: Vec<String>,
    pub entrypoint: String,
    // TODO: Add params parsing later
}

#[derive(Debug)]
pub struct ParsedScript<'a> {
    pub header: PixelscriptHeader,
    pub lua: ParsedLua<'a>,
}

impl<'a> ParsedScript<'a> {
    pub fn from_lua(mut lua: ParsedLua<'a>) -> Result<Self> {
        log::debug!("Converting Lua to pixelscript");

        // Create transformer that will extract header and check features
        let mut transformer = ScriptTransformer::new();

        // Transform the AST (extracts header, checks for invalid features)
        lua.ast.descend_mut(&mut transformer);

        // Check if we found errors
        if !transformer.errors.is_empty() {
            let error_msg = transformer
                .errors
                .iter()
                .map(|e| format!("  - {}", e))
                .collect::<Vec<_>>()
                .join("\n");
            bail!("Unsupported Lua features detected:\n{}", error_msg);
        }

        // Extract the header (must have been found)
        let header = transformer.header.ok_or_else(|| {
            anyhow!("Missing 'pixelscript = {{...}}' header as first statement in file")
        })?;

        log::info!(
            "Found pixelscript: name='{}', modules={:?}, entrypoint='{}'",
            header.name,
            header.modules,
            header.entrypoint
        );

        Ok(Self { header, lua })
    }
}

struct ScriptTransformer {
    header: Option<PixelscriptHeader>,
    errors: Vec<String>,
    in_function_depth: usize,
    header_extracted: bool,
}

impl ScriptTransformer {
    fn new() -> Self {
        Self {
            header: None,
            errors: Vec::new(),
            in_function_depth: 0,
            header_extracted: false,
        }
    }

    fn error(&mut self, msg: String) {
        self.errors.push(msg);
    }
}

impl<'a> VisitorMut<'a> for ScriptTransformer {
    fn visit_block(&mut self, block: &mut Block<'a>) {
        // Only process the top-level block specially
        if !self.header_extracted {
            self.header_extracted = true;

            // Check if first statement is pixelscript header
            if let Some(first_stmt) = block.statements.first_mut() {
                if let Statement::Assignment(assign) = first_stmt {
                    if assign.vars.pairs.len() == 1 {
                        // Check if target is a simple name "pixelscript"
                        if let Some(var) = assign.vars.pairs.first() {
                            if let Var::Name(name) = &var.0 {
                                if token_to_str(&name.0.token.value) == "pixelscript" {
                                    // Extract the header
                                    if let Some(expr_pair) = assign.exprs.pairs.first() {
                                        match parse_pixelscript_table(&expr_pair.0) {
                                            Ok(header) => {
                                                self.header = Some(header);
                                            }
                                            Err(e) => {
                                                self.error(format!("Failed to parse pixelscript header: {}", e));
                                            }
                                        }
                                    }

                                    // Now visit the remaining statements for feature checking
                                    for stmt in block.statements.iter_mut().skip(1) {
                                        stmt.descend_mut(self);
                                    }

                                    return;
                                }
                            }
                        }
                    }
                }
            }

            // If we get here, first statement wasn't pixelscript header
            self.error("pixelscript header must be the first statement in the file".to_string());
        }

        // For nested blocks, use default traversal
        for stmt in &mut block.statements {
            stmt.descend_mut(self);
        }
    }

    fn visit_stmt(&mut self, stmt: &mut Statement<'a>) {
        match stmt {
            Statement::Assignment(_) => {
                // Assignments are OK - but visit expressions
            }
            Statement::Block(_) => {
                // do...end blocks are OK
            }
            Statement::FunctionCall(_) => {
                // Function calls need to be checked separately
            }
            Statement::FunctionDeclaration(_func) => {
                // Function declarations are OK at top level
                // Check for nested functions
                if self.in_function_depth > 0 {
                    self.error("Nested function declarations (closures) are not supported".to_string());
                }
                // TODO: Check for method definitions once we understand the FunctionDeclarationStat structure
            }
            Statement::LocalDeclaration(local) => {
                // Check if it's a local function by checking if definition exists with exprs
                if let Some(definition) = &local.definition {
                    // Check if any of the expressions are function expressions
                    for expr_pair in &definition.exprs.pairs {
                        if matches!(&*expr_pair.0, Expr::Function(_)) {
                            if self.in_function_depth > 0 {
                                self.error("Nested local functions (closures) are not supported".to_string());
                            }
                        }
                    }
                }
                // local assignments are OK
            }
            Statement::If(_) => {
                // if statements are OK
            }
            Statement::For(for_stat) => {
                match &*for_stat {
                    ForStat::Generic(_) => {
                        self.error("Generic for loops (for k,v in ...) are not supported".to_string());
                    }
                    ForStat::Numerical(_) => {
                        // Numerical for loops are OK
                    }
                }
            }
            Statement::Repeat(_) => {
                // repeat...until is OK
            }
            Statement::While(_) => {
                // while loops are OK
            }
            Statement::Break(_) => {
                // break is OK
            }
            Statement::Return(ret) => {
                // Check for multiple returns
                if ret.exprs.pairs.len() > 1 {
                    self.error("Multiple return values are not supported".to_string());
                }
            }
            Statement::Goto(_) => {
                self.error("Goto statements are not supported".to_string());
            }
            Statement::Label(_) => {
                self.error("Label statements are not supported".to_string());
            }
            Statement::Empty(_) => {
                // Empty statements are OK
            }
        }

        // Continue visiting children (default traversal)
        stmt.descend_mut(self);
    }

    fn visit_expr(&mut self, expr: &mut Expr<'a>) {
        match expr {
            Expr::TableConstructor(_) => {
                // Tables are not allowed (except pixelscript header which is already handled)
                self.error("Table constructors are not supported".to_string());
            }
            Expr::Function(_) => {
                // Anonymous functions (closures) are not allowed
                self.error("Anonymous functions (closures) are not supported".to_string());
            }
            _ => {}
        }

        // Continue visiting children
        expr.descend_mut(self);
    }

    fn visit_function_call(&mut self, call: &mut FunctionCall<'a>) {
        // Check the callee to see if it's a method call or regular call
        match &call.callee {
            FunctionCallee::Method { .. } => {
                self.error("Method calls with colon syntax (:) are not supported".to_string());
            }
            FunctionCallee::Expr(head)=> {
                // Check the function being called
                if let PrefixExpr::Var(var) = &**head {
                    if let Var::Name(name) = var {
                        let func_name = token_to_str(&name.0.token.value);

                        // Whitelist of allowed functions
                        let allowed_functions = vec!["sleep"];

                        // Check for forbidden standard library functions
                        let forbidden_prefixes = vec![
                            "load",
                            "loadfile",
                            "dofile",
                            "require",
                            "pcall",
                            "xpcall",
                            "coroutine",
                            "debug",
                            "io",
                            "os",
                            "package",
                            "string",
                            "table",
                            "math",
                            "utf8",
                            "collectgarbage",
                            "getmetatable",
                            "setmetatable",
                            "rawget",
                            "rawset",
                            "rawequal",
                            "rawlen",
                        ];
                        let func_str = func_name.as_ref();
                        let is_allowed = allowed_functions.contains(&func_str);
                        let is_builtin = func_name.contains('.') || forbidden_prefixes.contains(&func_str);

                        if is_builtin && !is_allowed {
                            self.error(format!(
                                "Standard library function '{}' is not supported",
                                func_name
                            ));
                        }
                    }
                }
            }
        }

        // Continue visiting children
        call.descend_mut(self);
    }

    fn visit_function_body(&mut self, body: &mut FunctionBody<'a>) {
        self.in_function_depth += 1;

        // Check for varargs (...)
        if body.vararg.is_some() {
            self.error("Variable argument functions (...) are not supported".to_string());
        }

        // Visit the function body
        body.descend_mut(self);

        self.in_function_depth -= 1;
    }
}

fn parse_pixelscript_table(expr: &Expr) -> Result<PixelscriptHeader> {
    // Extract table constructor from the expression
    let table = match expr {
        Expr::TableConstructor(table) => table,
        _ => bail!("pixelscript must be assigned a table"),
    };

    let mut name = None;
    let mut modules = None;
    let mut entrypoint = None;

    for field in &table.fields {
        // Check if this field has a key (record field like `name = "value"`)
        if let Some((table_key, _)) = &field.key {
            match table_key {
                TableKey::Name { key } => {
                    let key_str = token_to_str(&key.0.token.value);
                    match key_str.as_ref() {
                        "name" => {
                            name = Some(extract_string_value(&field.value)?);
                        }
                        "modules" => {
                            modules = Some(extract_string_array(&field.value)?);
                        }
                        "entrypoint" => {
                            entrypoint = Some(extract_string_value(&field.value)?);
                        }
                        "params" => {
                            // TODO: Parse params later
                            log::debug!("Skipping params parsing for now");
                        }
                        _ => {
                            log::warn!("Unknown pixelscript field: {}", key_str);
                        }
                    }
                }
                TableKey::Expr { .. } => {
                    // Expression keys like [expr] are not expected in pixelscript header
                }
            }
        }
    }

    Ok(PixelscriptHeader {
        name: name.ok_or_else(|| anyhow!("pixelscript.name is required"))?,
        modules: modules.ok_or_else(|| anyhow!("pixelscript.modules is required"))?,
        entrypoint: entrypoint.ok_or_else(|| anyhow!("pixelscript.entrypoint is required"))?,
    })
}

fn extract_string_value(expr: &Expr) -> Result<String> {
    match expr {
        Expr::String(s) => {
            // StringLit contains a token, extract the string value from it
            let token_str = token_to_str(&s.0.token.value);
            // Remove quotes
            Ok(token_str.trim_matches('"').trim_matches('\'').to_string())
        }
        _ => bail!("Expected string value, got {:?}", expr),
    }
}

fn extract_string_array(expr: &Expr) -> Result<Vec<String>> {
    let table = match expr {
        Expr::TableConstructor(table) => table,
        _ => bail!("Expected table for array"),
    };

    let mut strings = Vec::new();
    for field in &table.fields {
        // List fields have no key (like {value1, value2, value3})
        if field.key.is_none() {
            strings.push(extract_string_value(&field.value)?);
        }
    }

    Ok(strings)
}
