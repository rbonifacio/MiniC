//! Abstract Syntax Tree (AST) node definitions for MiniC.
//!
//! # Overview
//!
//! This file defines every node type that can appear in a MiniC program:
//!
//! * [`Type`] — the MiniC type system (`int`, `float`, `bool`, `str`, arrays,
//!   functions, and the special `Any` used for polymorphic stdlib parameters).
//! * [`Literal`] — a constant value written directly in source code.
//! * [`Expr`] / [`ExprD`] — expressions (arithmetic, comparisons, calls, …).
//! * [`Statement`] / [`StatementD`] — statements (declarations, assignments,
//!   `if`, `while`, `return`, blocks).
//! * [`FunDecl`] — a single function declaration with its body.
//! * [`Program`] — the top-level container: a list of function declarations.
//!
//! Convenience type aliases pin the `Ty` parameter to either `()` or `Type`:
//! `UncheckedExpr`, `CheckedExpr`, `UncheckedProgram`, `CheckedProgram`, etc.
//!
//! # Design Decisions
//!
//! ## The `Ty` decoration parameter
//!
//! Every expression and statement node carries a `ty` field of type `Ty`.
//! This is a *generic type parameter* — a placeholder that the caller fills
//! in with a concrete type. Think of it like a slot that can hold different
//! things depending on the phase:
//!
//! * **Parser output** (`Ty = ()`): the slot is empty — the parser doesn't
//!   know types yet, so it stores the zero-size empty tuple `()`.
//! * **Type-checker output** (`Ty = Type`): the slot holds the inferred
//!   MiniC type, so every node knows whether it is an `Int`, `Float`, etc.
//!
//! Using a single parameterised definition avoids duplicating all the node
//! types and keeps the parser and type checker structurally in sync.
//!
//! ## `ExprD` wraps `Expr`
//!
//! `Expr<Ty>` is the *shape* of an expression (which operation it is).
//! `ExprD<Ty>` bundles that shape with its decoration: `{ exp: Expr<Ty>, ty: Ty }`.
//! Consumers always work with `ExprD` so that type information is always
//! available in one place.
//!
//! ## `Type::Any` for polymorphic stdlib parameters
//!
//! The built-in `print` function accepts any value type. Rather than adding
//! special-case logic throughout the type checker, the stdlib registers
//! `print` with a parameter type of `Type::Any`. The type checker's
//! compatibility check (`types_compatible`) treats `Any` as matching
//! everything, keeping the special case local to one function.

/// MiniC types: scalar, array, function, and Any (for polymorphic native params).
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
    Int,
    Float,
    Bool,
    Str,
    Array(Box<Type>),
    Fun(Vec<Type>, Box<Type>),
    Pointer(Box<Type>),
    /// Matches any type. Only used as a parameter type in native stdlib registrations.
    Any,
}

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

/// Expression with type decoration.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprD<Ty> {
    pub exp: Expr<Ty>,
    pub ty: Ty,
}

/// An expression: literals, identifiers, and composed operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr<Ty> {
    Literal(Literal),
    Ident(String),
    /// Unary minus (arithmetic)
    Neg(Box<ExprD<Ty>>),
    Add(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Sub(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Mul(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Div(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Eq(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Ne(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Lt(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Le(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Gt(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Ge(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Not(Box<ExprD<Ty>>),
    And(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    Or(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    /// Function call: name(args)
    Call {
        name: String,
        args: Vec<ExprD<Ty>>,
    },
    /// Array literal: [ expr, expr, ... ]
    ArrayLit(Vec<ExprD<Ty>>),
    /// Index expression: `base[index]`
    Index {
        base: Box<ExprD<Ty>>,
        index: Box<ExprD<Ty>>,
    },
    /// Pointer operations
    AddrOf(Box<ExprD<Ty>>),
    Deref(Box<ExprD<Ty>>),
}

/// Statement with type decoration.
#[derive(Debug, Clone, PartialEq)]
pub struct StatementD<Ty> {
    pub stmt: Statement<Ty>,
    pub ty: Ty,
}

/// A statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement<Ty> {
    /// Variable declaration with initialization: `int x = expr`.
    Decl {
        name: String,
        ty: Type,
        init: Box<ExprD<Ty>>,
    },
    Assign {
        target: Box<ExprD<Ty>>,
        value: Box<ExprD<Ty>>,
    },
    /// Block of statements: `{ stmt* }`
    Block {
        seq: Vec<StatementD<Ty>>,
    },
    Call {
        name: String,
        args: Vec<ExprD<Ty>>,
    },
    If {
        cond: Box<ExprD<Ty>>,
        then_branch: Box<StatementD<Ty>>,
        else_branch: Option<Box<StatementD<Ty>>>,
    },
    While {
        cond: Box<ExprD<Ty>>,
        body: Box<StatementD<Ty>>,
    },
    /// Return statement: `return [expr]`.
    Return(Option<Box<ExprD<Ty>>>),
}

/// A typed parameter: (name, type).
pub type Param = (String, Type);

/// A function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FunDecl<Ty> {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Box<StatementD<Ty>>,
}

/// A complete MiniC program: function declarations only. Execution starts at `main`.
#[derive(Debug, Clone, PartialEq)]
pub struct Program<Ty> {
    pub functions: Vec<FunDecl<Ty>>,
}

// Type synonyms for checked and unchecked phases.
pub type UncheckedExpr = ExprD<()>;
pub type CheckedExpr = ExprD<Type>;
pub type UncheckedStmt = StatementD<()>;
pub type CheckedStmt = StatementD<Type>;
pub type UncheckedFunDecl = FunDecl<()>;
pub type CheckedFunDecl = FunDecl<Type>;
pub type UncheckedProgram = Program<()>;
pub type CheckedProgram = Program<Type>;
