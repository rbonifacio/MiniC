//! Tree-walking interpreter for MiniC.
//!
//! # Overview
//!
//! This module is the final stage of the MiniC pipeline. It takes a
//! [`CheckedProgram`] — guaranteed by the
//! type system to be well-typed — and executes it, starting at `main`.
//!
//! Public entry point:
//!
//! * [`interpret`] — sets up the initial environment (stdlib functions +
//!   user-defined functions), then calls `main` with no arguments.
//!
//! Internal structure:
//!
//! * [`eval_expr`] — evaluates expressions to [`Value`]s.
//! * [`exec_stmt`] — executes statements, driving `eval_expr` as needed.
//! * [`value`] — defines the runtime value type and error type.
//!
//! # Design Decisions
//!
//! ## Tree-walking interpretation
//!
//! The interpreter directly recurses over the AST without compiling it to
//! any intermediate bytecode or machine code first. For every expression node
//! it encounters, it evaluates it to a `Value`; for every statement node, it
//! executes it. This is called *tree-walking* (or *AST-interpreting*).
//!
//! The approach was chosen for simplicity: the code closely mirrors the
//! language semantics and is easy to follow. The trade-off is performance —
//! a bytecode VM or native code compiler would run MiniC programs much
//! faster. For a teaching language with small programs, speed is not a
//! priority.
//!
//! ## Functions stored alongside variables in one environment
//!
//! Before execution begins, both stdlib functions (e.g., `print`, `sqrt`)
//! and user-defined functions are registered in the same
//! [`Environment<Value>`](crate::environment::Environment) as
//! `Value::Fn(...)` bindings. There is no separate function table. This
//! means function calls and variable lookups use the same mechanism, keeping
//! the interpreter uniform. It also means a function name can be shadowed by
//! a local variable — a deliberate design choice for simplicity.
//!
//! ## `eval_expr` / `exec_stmt` decomposition
//!
//! Expressions always produce a value; statements perform effects (declare
//! variables, branch, loop) and may or may not produce a value (only
//! `return` does). Splitting the interpreter into two functions — one for
//! each syntactic category — makes this distinction explicit in the code and
//! avoids having to handle the "no value" case inside expression evaluation.

pub mod eval_expr;
pub mod exec_stmt;
pub mod value;

use crate::environment::Environment;
use crate::ir::ast::CheckedProgram;
use crate::stdlib::NativeRegistry;

use eval_expr::eval_call;
use value::{FnValue, RuntimeError, Value};

/// Interpret a type-checked MiniC program, starting execution at `main`.
pub fn interpret(program: &CheckedProgram) -> Result<(), RuntimeError> {
    let mut env = Environment::<Value>::new();

    // Register native stdlib functions as Value::Fn(FnValue::Native) bindings.
    let registry = NativeRegistry::default();
    for (name, entry) in registry.iter() {
        env.declare(name.clone(), Value::Fn(FnValue::Native(entry.func)));
    }

    // Register user-defined functions as Value::Fn(FnValue::UserDefined) bindings.
    for fun in &program.functions {
        env.declare(fun.name.clone(), Value::Fn(FnValue::UserDefined(fun.clone())));
    }

    if env.get("main").is_none() {
        return Err(RuntimeError::new("no 'main' function found"));
    }

    eval_call("main", vec![], &mut env)?;
    Ok(())
}
