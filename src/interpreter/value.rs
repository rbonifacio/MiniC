//! Runtime value types for the MiniC interpreter.
//!
//! # Overview
//!
//! Defines the types that the interpreter works with at runtime:
//!
//! * [`Value`] — a runtime value: an integer, float, boolean, string, array,
//!   void, or function.
//! * [`FnValue`] — the two kinds of callable: a user-defined MiniC function
//!   or a native Rust function.
//! * [`NativeFn`] — a type alias for the signature of a native function.
//! * [`RuntimeError`] — an error produced during interpretation (e.g.,
//!   division by zero, out-of-bounds array access, undefined variable).
//!
//! # Design Decisions
//!
//! ## `Value` as a Rust enum
//!
//! In Rust, an `enum` can have *variants* that each carry different data.
//! `Value` uses this to represent every possible runtime value in a single
//! type:
//!
//! ```text
//! Value::Int(42)          — an integer
//! Value::Float(3.14)      — a floating-point number
//! Value::Bool(true)       — a boolean
//! Value::Str("hi")        — a string
//! Value::Array([...])     — a list of Values
//! Value::Void             — no value (returned by void functions)
//! Value::Fn(FnValue::...) — a callable function
//! ```
//!
//! This is the idiomatic Rust approach to *tagged unions* — a value that can
//! be one of several shapes. Using `match` on a `Value` forces the code to
//! handle every possible shape, which prevents runtime surprises.
//!
//! ## `FnValue` unifying user-defined and native functions
//!
//! Both MiniC functions and Rust-implemented stdlib functions are stored as
//! `Value::Fn(FnValue)`. `FnValue` has two variants:
//!
//! * `FnValue::UserDefined(CheckedFunDecl)` — holds the full AST of the
//!   function; the interpreter re-enters `exec_stmt` to run its body.
//! * `FnValue::Native(NativeFn)` — holds a *function pointer*, i.e., a
//!   reference to a specific Rust function. Calling it just calls that Rust
//!   function directly.
//!
//! Storing both in the same `Value::Fn` variant means the call site in
//! `eval_expr` does not need to know in advance whether a function is
//! user-defined or native — `match` handles the dispatch.
//!
//! ## `NativeFn` as a function pointer type
//!
//! ```rust,ignore
//! pub type NativeFn = fn(Vec<Value>) -> Result<Value, RuntimeError>;
//! ```
//!
//! `NativeFn` is a *function pointer type*: a variable of this type holds
//! the address of a Rust function with the matching signature. It is defined
//! here (rather than in `stdlib`) to avoid a circular dependency: `stdlib`
//! needs `Value`, and `Value` needs to reference the callable type.

use std::fmt;

use crate::ir::ast::CheckedFunDecl;

/// A native (Rust-implemented) MiniC function. Defined here to avoid circular deps with stdlib.
pub type NativeFn = fn(Vec<Value>) -> Result<Value, RuntimeError>;

/// A function value: either a MiniC-defined function or a Rust-native implementation.
#[derive(Clone)]
pub enum FnValue {
    UserDefined(CheckedFunDecl),
    Native(NativeFn),
}

impl PartialEq for FnValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FnValue::UserDefined(a), FnValue::UserDefined(b)) => a == b,
            (FnValue::Native(a), FnValue::Native(b)) => (*a as usize) == (*b as usize),
            _ => false,
        }
    }
}

impl fmt::Debug for FnValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FnValue::UserDefined(decl) => write!(f, "UserDefined({})", decl.name),
            FnValue::Native(_) => write!(f, "Native(<fn ptr>)"),
        }
    }
}

/// Runtime value in the MiniC interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Struct {
        tag_name: String,
        fields: std::collections::HashMap<String, Value>,
    },
    Union {
        tag_name: String,
        active_field: String,
        value: Box<Value>,
    },
    Enum {
        tag_name: String,
        value: i64,
    },
    Void,
    Fn(FnValue),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(x) => write!(f, "{}", x),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Void => write!(f, "void"),
            Value::Array(elems) => {
                write!(f, "[")?;
                for (i, v) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Struct { tag_name, .. } => write!(f, "<struct {}>", tag_name),
            Value::Union {
                tag_name,
                active_field,
                ..
            } => write!(f, "<union {} active:{}>", tag_name, active_field),
            Value::Enum { tag_name, value } => write!(f, "<enum {}={}>", tag_name, value),
            Value::Fn(_) => write!(f, "<function>"),
        }
    }
}

/// A runtime error produced during interpretation.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
