//! Abstract syntax tree for MiniC.

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// An expression: literals, identifiers, and composed operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    /// Unary minus (arithmetic)
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    /// Function call: name(args)
    Call { name: String, args: Vec<Expr> },
}

/// A function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FunDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Stmt>,
}

/// A statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign { target: String, value: Box<Expr> },
    /// Block of statements: `{ stmt ; stmt ; ... }`
    Block { seq: Vec<Stmt> },
    Call { name: String, args: Vec<Expr> },
    If {
        cond: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While { cond: Box<Expr>, body: Box<Stmt> },
}

/// A complete MiniC program: function declarations and main body.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<FunDecl>,
    pub body: Vec<Stmt>,
}
