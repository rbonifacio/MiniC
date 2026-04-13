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
    CheckedExpr, CheckedStmt, Expr, Member, Statement, TagType, TaggedTypeDecl, Type,
};

use std::collections::HashMap;

use super::eval_expr::{eval_call, eval_expr};
use super::value::{RuntimeError, Value};

type TaggedRuntimeTable = HashMap<(TagType, String), TaggedTypeDecl>;

/// `None` = normal fall-through; `Some(v)` = early return with value.
pub type ExecResult = Result<Option<Value>, RuntimeError>;

/// Execute a checked statement. Returns `Some(v)` if a `return` was hit.
pub fn exec_stmt(
    stmt: &CheckedStmt,
    env: &mut Environment<Value>,
    tagged_table: &TaggedRuntimeTable,
) -> ExecResult {
    match &stmt.stmt {
        // --- Variable declaration ---
        Statement::Decl { name, ty, init } => {
            let init_val = eval_expr(init, env, tagged_table)?;
            let stored = match ty {
                Type::Tagged { tag_type, tag_name } => {
                    build_tagged_value(tag_type, tag_name, init_val, tagged_table)?
                }
                _ => init_val,
            };
            env.declare(name.clone(), stored);
            Ok(None)
        }

        // --- Assignment ---
        Statement::Assign { target, value } => {
            let val = eval_expr(value, env, tagged_table)?;
            assign_lvalue(target, val, env, tagged_table)?;
            Ok(None)
        }

        // --- Block ---
        // Only remove variables declared inside the block on exit.
        // Assignments to outer-scope variables must persist (e.g., loop counters).
        Statement::Block { seq } => {
            let outer_keys = env.names();
            for s in seq {
                if let Some(ret) = exec_stmt(s, env, tagged_table)? {
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
        } => match eval_expr(cond, env, tagged_table)? {
            Value::Bool(true) => exec_stmt(then_branch, env, tagged_table),
            Value::Bool(false) => {
                if let Some(eb) = else_branch {
                    exec_stmt(eb, env, tagged_table)
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
            match eval_expr(cond, env, tagged_table)? {
                Value::Bool(true) => {
                    if let Some(ret) = exec_stmt(body, env, tagged_table)? {
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
            let val = eval_expr(expr, env, tagged_table)?;
            Ok(Some(val))
        }
        Statement::Return(None) => Ok(Some(Value::Void)),

        // --- Statement-level function call ---
        Statement::Call { name, args } => {
            let arg_vals: Result<Vec<Value>, RuntimeError> = args
                .iter()
                .map(|a| eval_expr(a, env, tagged_table))
                .collect();
            eval_call(name, arg_vals?, env, tagged_table)?;
            Ok(None)
        }
    }
}

/// Assign `val` to the lvalue described by `target`.
fn assign_lvalue(
    target: &CheckedExpr,
    val: Value,
    env: &mut Environment<Value>,
    tagged_table: &TaggedRuntimeTable,
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
            let idx = match eval_expr(index, &mut *env, tagged_table)? {
                Value::Int(i) => i as usize,
                v => {
                    return Err(RuntimeError::new(format!(
                        "array index must be int, got: {}",
                        v
                    )))
                }
            };
            assign_index(base, idx, val, env, tagged_table)
        }
        Expr::Member { base, member } => assign_member(base, member, val, env, tagged_table),
        _ => Err(RuntimeError::new("invalid assignment target".to_string())),
    }
}

/// Recursively assign into a (possibly nested) array lvalue.
fn assign_index(
    base: &CheckedExpr,
    idx: usize,
    val: Value,
    env: &mut Environment<Value>,
    tagged_table: &TaggedRuntimeTable,
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
            let inner_idx = match eval_expr(inner_index, env, tagged_table)? {
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
    _tagged_table: &TaggedRuntimeTable,
) -> Result<(), RuntimeError> {
    match &base.exp {
        Expr::Ident(name) => {
            let current = env
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("undefined variable '{}'", name)))?;
            let updated = match current {
                Value::Struct {
                    tag_name,
                    mut fields,
                } => {
                    fields.insert(member.to_string(), val);
                    Value::Struct { tag_name, fields }
                }
                Value::Union { tag_name, .. } => Value::Union {
                    tag_name,
                    active_field: member.to_string(),
                    value: Box::new(val),
                },
                other => {
                    return Err(RuntimeError::new(format!(
                        "cannot assign member on non-tagged value: {}",
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

fn build_tagged_value(
    tag_type: &TagType,
    tag_name: &str,
    init_val: Value,
    tagged_table: &TaggedRuntimeTable,
) -> Result<Value, RuntimeError> {
    let decl = tagged_table
        .get(&(tag_type.clone(), tag_name.to_string()))
        .ok_or_else(|| {
            RuntimeError::new(format!(
                "unknown tagged type at runtime: {:?} {}",
                tag_type, tag_name
            ))
        })?;

    match tag_type {
        TagType::Struct => {
            let mut fields = HashMap::new();
            for member in &decl.members {
                if let Member::Field(field) = member {
                    fields.insert(
                        field.name.clone(),
                        default_value_for_type(&field.ty, tagged_table)?,
                    );
                }
            }
            Ok(Value::Struct {
                tag_name: tag_name.to_string(),
                fields,
            })
        }
        TagType::Union => {
            let first_field = decl
                .members
                .iter()
                .find_map(|member| match member {
                    Member::Field(field) => Some(field),
                    _ => None,
                })
                .ok_or_else(|| {
                    RuntimeError::new(format!("union {} has no fields at runtime", tag_name))
                })?;

            let coerced = coerce_value_to_type(init_val, &first_field.ty)?;
            Ok(Value::Union {
                tag_name: tag_name.to_string(),
                active_field: first_field.name.clone(),
                value: Box::new(coerced),
            })
        }
        TagType::Enum => {
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
                tag_name: tag_name.to_string(),
                value: numeric,
            })
        }
    }
}

fn default_value_for_type(
    ty: &Type,
    tagged_table: &TaggedRuntimeTable,
) -> Result<Value, RuntimeError> {
    match ty {
        Type::Unit => Ok(Value::Void),
        Type::Int => Ok(Value::Int(0)),
        Type::Float => Ok(Value::Float(0.0)),
        Type::Bool => Ok(Value::Bool(false)),
        Type::Str => Ok(Value::Str(String::new())),
        Type::Array(_) => Ok(Value::Array(vec![])),
        Type::Tagged { tag_type, tag_name } => {
            build_tagged_value(tag_type, tag_name, Value::Int(0), tagged_table)
        }
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
