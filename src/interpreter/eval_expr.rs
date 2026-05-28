//! Expression evaluator for the MiniC interpreter.
//!
//! # Overview
//!
//! Exposes two public functions:
//!
//! * [`eval_expr`] — recursively evaluates a [`CheckedExpr`] to a [`Value`].
//!   This is the workhorse of the interpreter: every operator, literal,
//!   identifier lookup, array construction, and index access is handled here.
//! * [`eval_call`] — dispatches a function call by name. Called both from
//!   `eval_expr` (when a call appears inside an expression) and from
//!   `exec_stmt` (when a call is used as a statement).
//!
//! # Design Decisions
//!
//! ## Recursive evaluation mirrors the recursive AST
//!
//! `eval_expr` is a recursive function: to evaluate `Add(left, right)`, it
//! calls itself on `left` and `right`, then adds the resulting `Value`s.
//! This mirrors the recursive structure of the AST and is the defining
//! characteristic of tree-walking interpretation. Each AST node type has a
//! corresponding `match` arm.
//!
//! ## Short-circuit evaluation for `and` / `or`
//!
//! `and` and `or` do *not* eagerly evaluate both operands. For `and`, if the
//! left side is `false` the right side is never evaluated; for `or`, if the
//! left side is `true` the right side is skipped. This matches the behaviour
//! of these operators in most languages and is implemented simply by
//! returning early inside the `match` arm rather than calling `eval_expr` on
//! the second operand unconditionally.
//!
//! ## Function call scoping via `snapshot` / `restore`
//!
//! When `eval_call` calls a user-defined function, it snapshots the entire
//! environment, binds the arguments to the parameter names, runs the body,
//! then restores the snapshot. This gives the callee its own scope while
//! automatically cleaning up afterward — no matter whether the function
//! returns normally or early. See [`Environment`]
//! for more detail on this mechanism.

use crate::environment::Environment;
use crate::ir::ast::{CheckedExpr, Expr, Literal};

use super::exec_stmt::exec_stmt;
use super::value::{FnValue, RuntimeError, Value};

/// Evaluate a checked expression to a runtime value.
pub fn eval_expr(expr: &CheckedExpr, env: &mut Environment<Value>) -> Result<Value, RuntimeError> {
    match &expr.exp {
        Expr::Literal(lit) => Ok(eval_literal(lit)),

        Expr::Ident(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(format!("undefined variable '{}'", name))),

        Expr::Neg(inner) => match eval_expr(inner, env)? {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(x) => Ok(Value::Float(-x)),
            v => Err(RuntimeError::new(format!(
                "cannot negate non-numeric value: {}",
                v
            ))),
        },

        Expr::Add(l, r) => numeric_binop(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a + b, |a, b| a + b),
        Expr::Sub(l, r) => numeric_binop(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a - b, |a, b| a - b),
        Expr::Mul(l, r) => numeric_binop(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a * b, |a, b| a * b),
        Expr::Div(l, r) => numeric_binop(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a / b, |a, b| a / b),

        Expr::Lt(l, r) => numeric_cmp(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a < b, |a, b| a < b),
        Expr::Le(l, r) => numeric_cmp(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a <= b, |a, b| a <= b),
        Expr::Gt(l, r) => numeric_cmp(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a > b, |a, b| a > b),
        Expr::Ge(l, r) => numeric_cmp(eval_expr(l, env)?, eval_expr(r, env)?, |a, b| a >= b, |a, b| a >= b),

        Expr::Eq(l, r) => {
            let lv = eval_expr(l, env)?;
            let rv = eval_expr(r, env)?;
            Ok(Value::Bool(values_equal(&lv, &rv)))
        }
        Expr::Ne(l, r) => {
            let lv = eval_expr(l, env)?;
            let rv = eval_expr(r, env)?;
            Ok(Value::Bool(!values_equal(&lv, &rv)))
        }

        Expr::Not(inner) => match eval_expr(inner, env)? {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            v => Err(RuntimeError::new(format!(
                "expected bool for '!', got: {}",
                v
            ))),
        },
        Expr::And(l, r) => {
            let lv = eval_expr(l, env)?;
            match lv {
                Value::Bool(false) => Ok(Value::Bool(false)),
                Value::Bool(true) => eval_expr(r, env),
                v => Err(RuntimeError::new(format!(
                    "expected bool for 'and', got: {}",
                    v
                ))),
            }
        }
        Expr::Or(l, r) => {
            let lv = eval_expr(l, env)?;
            match lv {
                Value::Bool(true) => Ok(Value::Bool(true)),
                Value::Bool(false) => eval_expr(r, env),
                v => Err(RuntimeError::new(format!(
                    "expected bool for 'or', got: {}",
                    v
                ))),
            }
        }

        Expr::ArrayLit(elems) => {
            let vals: Result<Vec<Value>, RuntimeError> =
                elems.iter().map(|e| eval_expr(e, env)).collect();
            Ok(Value::Array(vals?))
        }

        Expr::Index { base, index } => {
            let base_val = eval_expr(base, env)?;
            let idx_val = eval_expr(index, env)?;
            match (base_val, idx_val) {
                (Value::Array(elems), Value::Int(i)) => {
                    let i = i as usize;
                    elems.into_iter().nth(i).ok_or_else(|| {
                        RuntimeError::new(format!("array index {} out of bounds", i))
                    })
                }
                (Value::Array(_), idx) => Err(RuntimeError::new(format!(
                    "array index must be int, got: {}",
                    idx
                ))),
                (base, _) => Err(RuntimeError::new(format!(
                    "cannot index non-array value: {}",
                    base
                ))),
            }
        }

        Expr::Call { name, args } => {
            let arg_vals: Result<Vec<Value>, RuntimeError> =
                args.iter().map(|a| eval_expr(a, env)).collect();
            eval_call(name, arg_vals?, env)
        }

        Expr::AddrOf(_) | Expr::Deref(_) => Err(RuntimeError::new(
            "address-of and dereference are not implemented in the interpreter yet",
        )),
    }
}

/// Dispatch a function call via the unified environment.
pub fn eval_call(
    name: &str,
    args: Vec<Value>,
    env: &mut Environment<Value>,
) -> Result<Value, RuntimeError> {
    match env.get(name).cloned() {
        Some(Value::Fn(FnValue::Native(f))) => (f)(args),
        Some(Value::Fn(FnValue::UserDefined(decl))) => {
            if args.len() != decl.params.len() {
                return Err(RuntimeError::new(format!(
                    "function '{}' expects {} arguments, got {}",
                    name,
                    decl.params.len(),
                    args.len()
                )));
            }
            let snapshot = env.snapshot();
            for ((param_name, _), val) in decl.params.iter().zip(args.into_iter()) {
                env.declare(param_name.clone(), val);
            }
            let result = exec_stmt(&decl.body, env)?;
            env.restore(snapshot);
            Ok(result.unwrap_or(Value::Void))
        }
        Some(_) => Err(RuntimeError::new(format!("'{}' is not a function", name))),
        None => Err(RuntimeError::new(format!("undefined function '{}'", name))),
    }
}

// --- Helpers ---

fn eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Int(n) => Value::Int(*n),
        Literal::Float(x) => Value::Float(*x),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Str(s) => Value::Str(s.clone()),
    }
}

fn numeric_binop(
    lv: Value,
    rv: Value,
    int_op: impl Fn(i64, i64) -> i64,
    float_op: impl Fn(f64, f64) -> f64,
) -> Result<Value, RuntimeError> {
    match (lv, rv) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(int_op(a, b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(a, b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(float_op(a as f64, b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(float_op(a, b as f64))),
        (l, r) => Err(RuntimeError::new(format!(
            "arithmetic requires numeric operands, got: {} and {}",
            l, r
        ))),
    }
}

fn numeric_cmp(
    lv: Value,
    rv: Value,
    int_cmp: impl Fn(i64, i64) -> bool,
    float_cmp: impl Fn(f64, f64) -> bool,
) -> Result<Value, RuntimeError> {
    match (lv, rv) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(int_cmp(a, b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(float_cmp(a, b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool(float_cmp(a as f64, b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(float_cmp(a, b as f64))),
        (l, r) => Err(RuntimeError::new(format!(
            "comparison requires numeric operands, got: {} and {}",
            l, r
        ))),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y)) => *x == (*y as f64),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        _ => false,
    }
}
