use std::fmt;

/// A span representing a location in the source code
pub type Span = std::ops::Range<usize>;

/// A value with associated source location
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

// ============================================================================
// Top-level Program Structure
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub metadata: Spanned<MetadataBlock>,
    pub block: Block,
    pub span: Span,
}

// ============================================================================
// Metadata Block (Header)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataBlock {
    pub name: Spanned<String>,  // The variable name (should be "pixelscript")
    pub table: Spanned<MetadataTable>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataTable {
    pub fields: Vec<Spanned<MetadataField>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataField {
    /// name = literal
    Literal {
        name: Spanned<String>,
        value: Spanned<Literal>,
    },
    /// name = { nested_fields }
    Table {
        name: Spanned<String>,
        table: Spanned<MetadataTable>,
    },
    /// name = (args...)  -- Function-call-like syntax
    Call {
        name: Spanned<String>,
        args: Vec<Spanned<Literal>>,
    },
    /// name = {item1, item2}  -- List syntax
    List {
        name: Spanned<String>,
        items: Vec<Spanned<Literal>>,
    },
}

// ============================================================================
// Literals
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}

// ============================================================================
// Lua Code Structures
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Spanned<Statement>>,
    pub return_stmt: Option<Box<Spanned<ReturnStmt>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    FunctionCall(FunctionCall),
    DoBlock(Block),
    While(WhileStmt),
    Repeat(RepeatStmt),
    If(IfStmt),
    ForNum(ForNumStmt),
    ForIn(ForInStmt),
    FunctionDef(FunctionDef),
    LocalFunctionDef(LocalFunctionDef),
    LocalDecl(LocalDecl),
    Break,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub var: Spanned<String>,
    pub value: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub func: Spanned<PrefixExpr>,
    pub args: Vec<Spanned<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Spanned<Expr>,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepeatStmt {
    pub block: Block,
    pub condition: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Spanned<Expr>,
    pub then_block: Block,
    pub elseif_branches: Vec<(Spanned<Expr>, Block)>,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForNumStmt {
    pub var: Spanned<String>,
    pub start: Spanned<Expr>,
    pub end: Spanned<Expr>,
    pub step: Option<Spanned<Expr>>,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForInStmt {
    pub var: Spanned<String>,
    pub iterator: Spanned<String>,  // Only Name allowed, not full expr
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDef {
    pub name: Spanned<String>,
    pub params: Vec<Spanned<String>>,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFunctionDef {
    pub name: Spanned<String>,
    pub params: Vec<Spanned<String>>,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalDecl {
    pub name: Spanned<String>,
    pub value: Option<Spanned<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Spanned<Expr>>,
}

// ============================================================================
// Expressions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Var(String),
    Index(IndexExpr),
    Parenthesized(Box<Spanned<Expr>>),
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    TableConstructor(TableConstructor),
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub base: Box<Spanned<PrefixExpr>>,
    pub index: Box<Spanned<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixExpr {
    Var(String),
    Parenthesized(Box<Spanned<Expr>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOp {
    pub left: Box<Spanned<Expr>>,
    pub op: Spanned<BinOp>,
    pub right: Box<Spanned<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOp {
    pub op: Spanned<UnOp>,
    pub operand: Box<Spanned<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableConstructor {
    pub fields: Vec<Spanned<TableField>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableField {
    /// [literal] = literal
    Indexed {
        key: Spanned<Literal>,
        value: Spanned<Literal>,
    },
    /// name = literal
    Named {
        name: Spanned<String>,
        value: Spanned<Literal>,
    },
    /// literal (positional)
    Value(Spanned<Literal>),
}

// ============================================================================
// Operators
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    // Comparison
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

// ============================================================================
// Helper methods
// ============================================================================

impl BinOp {
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne => 3,
            BinOp::Add | BinOp::Sub => 5,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 6,
            BinOp::Pow => 7,  // Right-associative, higher precedence
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, BinOp::Pow)
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Pow => "^",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "~=",
            BinOp::And => "and",
            BinOp::Or => "or",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnOp::Neg => "-",
            UnOp::Not => "not",
        };
        write!(f, "{}", s)
    }
}
