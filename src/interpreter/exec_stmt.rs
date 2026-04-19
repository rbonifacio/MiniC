//! Statement executor for the MiniC interpreter.
//!
//! # Overview
//!
//! Exposes one public function:
//!
//! * [`exec_stmt`] — executes a [`CheckedStmt`] and returns an [`ExecResult`]:
//!   * `Ok(None)` — the statement completed normally (no early return).
//!   * `Ok(Some(value))` — a `return` statement was hit; the value is
//!     propagated up to the caller (`eval_call` in `eval_expr`).
//!   * `Err(RuntimeError)` — a runtime error occurred.
//!
//! Also defines `ExecResult`, the return type alias used throughout.
//!
//! # Design Decisions
//!
//! ## `Option<Value>` to signal early return
//!
//! Statements do not inherently produce values, but a `return` statement
//! inside a function body must pass its value back through potentially many
//! levels of nested `exec_stmt` calls (blocks inside loops inside blocks,
//! etc.). Using `Option<Value>` as the success case of `ExecResult` encodes
//! this cleanly: `None` means "keep going", `Some(v)` means "stop and return
//! this value". Each `Block` and `While` arm checks for `Some` and
//! short-circuits immediately.
//!
//! ## Block scoping via `names` / `remove_new`
//!
//! When `exec_stmt` enters a `Block`, it records the set of names currently
//! bound in the environment (`env.names()`). When the block exits, it calls
//! `env.remove_new(&outer_keys)` to drop any name that was declared inside
//! the block. Crucially, this only removes *new* bindings — assignments to
//! variables declared in an outer scope (e.g., a loop counter) are preserved.
//! This gives MiniC correct lexical block scoping without a scope stack.

use crate::environment::Environment;
use crate::ir::ast::{
    AgtTypeMember, AgtTypeSpecifier, CheckedExpr, CheckedStmt, Expr, Statement, Type,
};

use super::eval_expr::{eval_call, eval_expr};
use super::value::{RuntimeError, Value};

use std::collections::HashMap;

/// `None` = normal fall-through; `Some(v)` = early return with value.
pub type ExecResult = Result<Option<Value>, RuntimeError>;

/// Execute a checked statement. Returns `Some(v)` if a `return` was hit.
pub fn exec_stmt(stmt: &CheckedStmt, env: &mut Environment<Value>) -> ExecResult {
    match &stmt.stmt {
        // --- Variable declaration ---
        Statement::Decl { name, ty, init } => {
            let init_val = eval_expr(init, env)?;
            let stored = match ty {
                Type::Aggregate {
                    specifier,
                    identifier,
                } => build_aggregate_value(specifier, identifier, init_val, env)?,
                _ => init_val,
            };
            env.declare(name.clone(), stored);
            Ok(None)
        }

        // --- Assignment ---
        Statement::Assign { target, value } => {
            let val = eval_expr(value, env)?;
            assign_lvalue(target, val, env)?;
            Ok(None)
        }

        // --- Block ---
        // Only remove variables declared inside the block on exit.
        // Assignments to outer-scope variables must persist (e.g., loop counters).
        Statement::Block { seq } => {
            let outer_keys = env.names();
            for s in seq {
                if let Some(ret) = exec_stmt(s, env)? {
                    env.remove_new(&outer_keys);
                    return Ok(Some(ret));
                }
            }
            env.remove_new(&outer_keys);
            Ok(None)
        }

        // --- If ---
        Statement::If {
            cond,
            then_branch,
            else_branch,
        } => match eval_expr(cond, env)? {
            Value::Bool(true) => exec_stmt(then_branch, env),
            Value::Bool(false) => {
                if let Some(eb) = else_branch {
                    exec_stmt(eb, env)
                } else {
                    Ok(None)
                }
            }
            v => Err(RuntimeError::new(format!(
                "if condition must be bool, got: {}",
                v
            ))),
        },

        // --- While ---
        Statement::While { cond, body } => loop {
            match eval_expr(cond, env)? {
                Value::Bool(true) => {
                    if let Some(ret) = exec_stmt(body, env)? {
                        return Ok(Some(ret));
                    }
                }
                Value::Bool(false) => return Ok(None),
                v => {
                    return Err(RuntimeError::new(format!(
                        "while condition must be bool, got: {}",
                        v
                    )))
                }
            }
        },

        // --- Return ---
        Statement::Return(Some(expr)) => {
            let val = eval_expr(expr, env)?;
            Ok(Some(val))
        }
        Statement::Return(None) => Ok(Some(Value::Void)),

        // --- Statement-level function call ---
        Statement::Call { name, args } => {
            let arg_vals: Result<Vec<Value>, RuntimeError> =
                args.iter().map(|a| eval_expr(a, env)).collect();
            eval_call(name, arg_vals?, env)?;
            Ok(None)
        }
    }
}

/// Assign `val` to the lvalue described by `target`.
fn assign_lvalue(
    target: &CheckedExpr,
    val: Value,
    env: &mut Environment<Value>,
) -> Result<(), RuntimeError> {
    match &target.exp {
        Expr::Ident(name) => {
            if env.set(name, val) {
                Ok(())
            } else {
                Err(RuntimeError::new(format!(
                    "assignment to undeclared variable '{}'",
                    name
                )))
            }
        }
        Expr::Index { base, index } => {
            let idx = match eval_expr(index, &mut *env)? {
                Value::Int(i) => i as usize,
                v => {
                    return Err(RuntimeError::new(format!(
                        "array index must be int, got: {}",
                        v
                    )))
                }
            };
            assign_index(base, idx, val, env)
        }
        Expr::Member { base, member } => assign_member(base, member, val, env),
        _ => Err(RuntimeError::new("invalid assignment target".to_string())),
    }
}

/// Recursively assign into a (possibly nested) array lvalue.
fn assign_index(
    base: &CheckedExpr,
    idx: usize,
    val: Value,
    env: &mut Environment<Value>,
) -> Result<(), RuntimeError> {
    match &base.exp {
        Expr::Ident(name) => {
            let arr = env
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("undefined variable '{}'", name)))?;
            match arr {
                Value::Array(mut elems) => {
                    if idx >= elems.len() {
                        return Err(RuntimeError::new(format!(
                            "array index {} out of bounds (len {})",
                            idx,
                            elems.len()
                        )));
                    }
                    elems[idx] = val;
                    env.set(name, Value::Array(elems));
                    Ok(())
                }
                v => Err(RuntimeError::new(format!(
                    "cannot index non-array value: {}",
                    v
                ))),
            }
        }
        Expr::Index {
            base: inner_base,
            index: inner_index,
        } => {
            let inner_idx = match eval_expr(inner_index, env)? {
                Value::Int(i) => i as usize,
                v => {
                    return Err(RuntimeError::new(format!(
                        "array index must be int, got: {}",
                        v
                    )))
                }
            };
            let outer_name = extract_ident_name(inner_base)?;
            let outer = env
                .get(&outer_name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("undefined variable '{}'", outer_name)))?;
            match outer {
                Value::Array(mut outer_elems) => {
                    if inner_idx >= outer_elems.len() {
                        return Err(RuntimeError::new(format!(
                            "array index {} out of bounds (len {})",
                            inner_idx,
                            outer_elems.len()
                        )));
                    }
                    match &mut outer_elems[inner_idx] {
                        Value::Array(ref mut inner_elems) => {
                            if idx >= inner_elems.len() {
                                return Err(RuntimeError::new(format!(
                                    "array index {} out of bounds (len {})",
                                    idx,
                                    inner_elems.len()
                                )));
                            }
                            inner_elems[idx] = val;
                        }
                        v => {
                            return Err(RuntimeError::new(format!(
                                "cannot index non-array value: {}",
                                v
                            )))
                        }
                    }
                    env.set(&outer_name, Value::Array(outer_elems));
                    Ok(())
                }
                v => Err(RuntimeError::new(format!(
                    "cannot index non-array value: {}",
                    v
                ))),
            }
        }
        _ => Err(RuntimeError::new("invalid assignment target".to_string())),
    }
}

fn assign_member(
    base: &CheckedExpr,
    member: &str,
    val: Value,
    env: &mut Environment<Value>,
) -> Result<(), RuntimeError> {
    match &base.exp {
        Expr::Ident(name) => {
            let current = env
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("undefined variable '{}'", name)))?;
            let updated = match current {
                Value::Struct {
                    identifier,
                    mut fields,
                } => {
                    fields.insert(member.to_string(), val);
                    Value::Struct { identifier, fields }
                }
                Value::Union { identifier, .. } => Value::Union {
                    identifier,
                    active_field: member.to_string(),
                    value: Box::new(val),
                },
                other => {
                    return Err(RuntimeError::new(format!(
                        "cannot assign member on non-aggregate value: {}",
                        other
                    )))
                }
            };
            env.set(name, updated);
            Ok(())
        }
        _ => Err(RuntimeError::new(
            "member assignment currently requires a simple variable base".to_string(),
        )),
    }
}

fn build_aggregate_value(
    specifier: &AgtTypeSpecifier,
    identifier: &str,
    init_val: Value,
    env: &Environment<Value>,
) -> Result<Value, RuntimeError> {
    let decl = env.aggregate_type(specifier, identifier).ok_or_else(|| {
        RuntimeError::new(format!(
            "unknown aggregate type at runtime: {:?} {}",
            specifier, identifier
        ))
    })?;

    match specifier {
        AgtTypeSpecifier::Struct => {
            let mut fields = HashMap::new();
            for member in &decl.members {
                if let AgtTypeMember::Field(field) = member {
                    fields.insert(field.name.clone(), default_value_for_type(&field.ty, env)?);
                }
            }
            Ok(Value::Struct {
                identifier: identifier.to_string(),
                fields,
            })
        }
        AgtTypeSpecifier::Union => {
            let first_field = decl
                .members
                .iter()
                .find_map(|member| match member {
                    AgtTypeMember::Field(field) => Some(field),
                    _ => None,
                })
                .ok_or_else(|| {
                    RuntimeError::new(format!("union {} has no fields at runtime", identifier))
                })?;

            let coerced = coerce_value_to_type(init_val, &first_field.ty)?;
            Ok(Value::Union {
                identifier: identifier.to_string(),
                active_field: first_field.name.clone(),
                value: Box::new(coerced),
            })
        }
        AgtTypeSpecifier::Enum => {
            let numeric = match init_val {
                Value::Int(n) => n,
                other => {
                    return Err(RuntimeError::new(format!(
                        "enum initializer must be integer, got {}",
                        other
                    )))
                }
            };

            Ok(Value::Enum {
                identifier: identifier.to_string(),
                value: numeric,
            })
        }
    }
}

fn default_value_for_type(ty: &Type, env: &Environment<Value>) -> Result<Value, RuntimeError> {
    match ty {
        Type::Unit => Ok(Value::Void),
        Type::Int => Ok(Value::Int(0)),
        Type::Float => Ok(Value::Float(0.0)),
        Type::Bool => Ok(Value::Bool(false)),
        Type::Str => Ok(Value::Str(String::new())),
        Type::Array(_) => Ok(Value::Array(vec![])),
        Type::Aggregate {
            specifier,
            identifier,
        } => build_aggregate_value(specifier, identifier, Value::Int(0), env),
        Type::Function { .. } | Type::Any => Err(RuntimeError::new(
            "cannot create default runtime value for this type",
        )),
    }
}

fn coerce_value_to_type(val: Value, ty: &Type) -> Result<Value, RuntimeError> {
    match (val, ty) {
        (Value::Int(n), Type::Int) => Ok(Value::Int(n)),
        (Value::Int(n), Type::Float) => Ok(Value::Float(n as f64)),
        (Value::Int(n), Type::Bool) => Ok(Value::Bool(n != 0)),
        (Value::Int(n), Type::Str) => Ok(Value::Str(n.to_string())),
        (Value::Float(x), Type::Float) => Ok(Value::Float(x)),
        (Value::Float(x), Type::Int) => Ok(Value::Int(x as i64)),
        (Value::Bool(b), Type::Bool) => Ok(Value::Bool(b)),
        (Value::Str(s), Type::Str) => Ok(Value::Str(s)),
        (other, _) => Err(RuntimeError::new(format!(
            "cannot coerce value {} to required type {:?}",
            other, ty
        ))),
    }
}

fn extract_ident_name(expr: &CheckedExpr) -> Result<String, RuntimeError> {
    match &expr.exp {
        Expr::Ident(name) => Ok(name.clone()),
        _ => Err(RuntimeError::new(
            "nested array assignment only supported for simple variable bases".to_string(),
        )),
    }
}
