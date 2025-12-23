
use rpled_pixelscript::ast::nodes::*;
use rpled_vm::ops::Op;

enum VariableRef {
    Heap(usize),
    Frame(usize),
}

enum ExpressionResultKind {
    ConstI16(i16),
    ConstF32(f32),
    Variable(VariableRef),
    ConstNil,
}
struct ExpressionResult {
    kind: ExpressionResultKind,
    used: bool,
}

impl ExpressionResult {
    fn push(&mut self, visitor: &mut CompilerVisitor) {
        self.used = true;
        match &self.kind {
            ExpressionResultKind::ConstI16(n) => {
                visitor.ops.push(Op::Push { value: *(n.cast_ref::<u16>()) });
            }
            ExpressionResultKind::ConstF32(x) => {
                todo!("Handle pushing f32 constant");
            }
            ExpressionResultKind::Variable(var_ref) => {
                todo!("Handle variable reference");
            }
            ExpressionResultKind::FrameRef(addr) => {
                todo!("Handle pushing frame reference");
            }
        }
    }

    fn mark_used(&mut self) {
        self.used = true;
    }
}

impl Drop for ExpressionResult {
    fn drop(&mut self) {
        if self.used {
            panic!("ExpressionResult was not pushed or marked used before being dropped");
        }
    }
}

impl From<ExpressionResultKind> for ExpressionResult {
    fn from(kind: ExpressionResultKind) -> Self {
        ExpressionResult { kind, used: false }
    }
}

struct CompilerVisitor {
    global_scope: Scope,
    local_scopes: Vec<Scope>,
   
   ops: Vec<Op>,
   heap: Vec<u8>,
   current_frame: Option<Vec<u16>>,
}

impl CompilerVisitor {
    fn new() -> Self {
        CompilerVisitor {
            global_scope: Scope { variables: vec![] },
            local_scopes: vec![],
            ops: vec![],
            heap: vec![],
            current_frame: None,
        }
    }

    fn enter_frame(&mut self) {
        self.current_frame = Some(vec![]);
    }

    fn reset_frame(&mut self) {
        self.current_frame = None;
    }

    fn enter_scope(&mut self) {
        self.local_scopes.push(Scope { variables: vec![] });
    }

    fn exit_scope(&mut self) {
        self.local_scopes.pop();
    }

    fn lookup_name(&self, name: &str) -> Option<VariableRef> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(var) = scope.resolve_name(name) {
                return Some(VariableRef::Frame(var))
            }
        }
        if let Some(var) = self.global_scope.resolve_name(name) {
            return Some(VariableRef::Heap(var))
        }
        None
    }

    fn get_variable_ref(&mut self, name: &str) -> (VariableRef, bool) {
        // Try to find the variable in local scopes first
        // If not found, check global scope
        // If still not found, create a new global variable
        if let Some(var_ref) = self.lookup_name(name) {
            (var_ref, false)
        } else {
            // Create new global variable
            let addr = self.global_scope.allocate(name.to_string());
            (VariableRef::Heap(addr), true)
        }
    }

    fn visit_program(&mut self, program: &Program) {
        self.visit_metadata(&program.metadata);
        self.visit_block(&program.block);
    }

    fn visit_block(&mut self, block: &Block) {
        self.enter_scope();
        for statement in &block.statements {
            self.visit_statement(statement);
        }
        self.exit_scope()
    }

    fn visit_expression(&mut self, expr: &Expression) -> ExpressionResult {
        match expr {
            Expression::Constant(value) => {
                self.visit_const_expr(value)
            }
            Expression::Variable(name) => {
                self.visit_var_expr(name)
            }
            Expression::BinaryOp { left, op, right } => {
                self.visit_binary_op_expr(left, op, right)
            }
            Expression::UnaryOp { op, expr } => {
                self.visit_unary_op_expr(op, expr)
            }
            Expression::FunctionCall { name, args } => {
                self.visit_function_call_expr(name, args)
            }
            Expression::TableDef(fields) => {
                self.visit_table_def_expr(fields)
            }
        }
    }

    // ---- Expression Visitors ----
    fn visit_const_expr(&mut self, value: &Constant) -> ExpressionResult {
        match value {
            Constant::Num(n) => {
                ExpressionResultKind::ConstI16(*n).into()
            }
            Constant::Float(x) => {
                ExpressionResultKind::ConstF32(*x).into()
            }
            Constant::String(s) => {
                todo!("Implement string constant visitor")
            }
            Constant::True => {
                ExpressionResultKind::ConstI16(1).into()
            }
            Constant::False => {
                ExpressionResultKind::ConstI16(0).into()
            }
            Constant::Nil => {
                ExpressionResultKind::ConstNil.into()
            }
        }
    }

    fn visit_var_expr(&mut self, name: &String) -> ExpressionResult {
        todo!("Implement variable expression visitor")
    }

    fn visit_binary_op_expr(&mut self, left: &Expression, op: &String, right: &Expression) -> ExpressionResult {
        todo!("Implement binary operation expression visitor")
    }

    fn visit_unary_op_expr(&mut self, op: &String, expr: &Expression) -> ExpressionResult {
        todo!("Implement unary operation expression visitor")
    }

    fn visit_function_call_expr(&mut self, name: &String, args: &Vec<Expression>) -> ExpressionResult {
        todo!("Implement function call expression visitor")
    }

    fn visit_table_def_expr(&mut self, fields: &Vec<(Expression, Expression)>) -> ExpressionResult {
        todo!("Implement table definition expression visitor")
    }

    // ---- Statement Visitors ----

    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Assignment { target, value, local } => self.visit_stmt_assignment(target, value, *local),
            Statement::LocalDeclaration { name } => self.visit_stmt_local_declaration(name),
            Statement::FunctionCall {name , args } => self.visit_stmt_function_call(name, args),
            Statement::Block(block) => self.visit_stmt_block(block.as_ref()),
            Statement::WhileLoop {cond, block} => self.visit_stmt_while_loop(cond, block),
            Statement::RepeatLoop {cond, block} => self.visit_stmt_repeat_loop(cond, block),
            Statement::IfStmt {if_part, else_if_part, else_part} => self.visit_stmt_if_stmt(if_part, else_if_part, else_part),
            Statement::ForIn {name, iter, block} => self.visit_stmt_for_in(name, iter, block),
            Statement::ForNum {name, start, end, step, block} => self.visit_stmt_for_num(name, start, end, step, block),
            Statement::FunctionDef {name, params, block, local} => self.visit_stmt_function_def(name, params, block, local),
            Statement::Return { expr} => self.visit_stmt_return(expr),
            Statement::Break => self.visit_stmt_break(),
            Statement::Comment { text } => self.visit_stmt_comment(text),
        }
    }

    fn visit_stmt_assignment(&mut self, target: &String, value: &Expression, local: bool) {
        // Visit the value expression first
        let expr_result = self.visit_expression(value);
        let (target, is_new) = if local {
            (VariableRef::Local(self.alloc_local_variable(target)), true)
        } else {
            self.get_variable_ref(target)
        };
        
        match expr_result.kind {


        }
    }


}}