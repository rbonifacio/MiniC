//! Type checker implementation for MiniC.
//!
//! # Overview
//!
//! Provides [`type_check`], which walks an [`UncheckedProgram`] and either
//! returns a [`CheckedProgram`] (every node annotated with its [`Type`]) or
//! a [`TypeError`] describing the first violation found.
//!
//! Also defines [`TypeError`], the error type returned on failure.
//!
//! # Design Decisions
//!
//! ## Using `Environment<Type>` for variable tracking
//!
//! The type checker stores the *declared type* of every in-scope name in an
//! [`Environment<Type>`](crate::environment::Environment). Here `Type` is the
//! MiniC type (e.g., `Type::Int`), not a Rust type. This is the same
//! `Environment` struct used by the interpreter — but instantiated with
//! `Type` instead of `Value`. Functions are also stored in this environment
//! as `Type::Fun(param_types, return_type)`, so the same lookup mechanism
//! handles both variable and function name resolution.
//!
//! ## Function signatures registered before bodies are checked
//!
//! All function signatures are added to the environment before any function
//! body is checked. This allows functions to call each other (mutual
//! recursion) without requiring forward declarations. A `fn_snapshot` of the
//! function-only environment is taken after this step and restored at the
//! start of each function body check, ensuring variable bindings from one
//! function do not leak into another.
//!
//! ## Block scoping via `snapshot` / `restore`
//!
//! When the type checker enters a block statement, it takes a snapshot of the
//! current environment. When the block exits (normally or via early return),
//! it restores the snapshot, discarding any variables declared inside. This
//! correctly implements lexical block scoping without a separate scope-stack
//! data structure.
//!
//! ## `Type::Any` and `types_compatible`
//!
//! The `types_compatible` function implements MiniC's assignability rules,
//! including `Int`↔`Float` coercion and the `Any` wildcard used by `print`.
//! Centralising compatibility logic here means all callers (declaration,
//! assignment, call-argument checking) share one consistent definition.

use std::collections::HashMap;

use crate::environment::Environment;
use crate::ir::ast::{
    CheckedExpr, CheckedFunDecl, CheckedProgram, CheckedStmt, Expr, ExprD, FunDecl, Literal,
    Member, Program, Statement, StatementD, TagType, TaggedTypeDecl, Type, UncheckedExpr,
    UncheckedFunDecl, UncheckedProgram, UncheckedStmt,
};
use crate::stdlib::NativeRegistry;

/// A type error reported by the type checker.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub message: String,
}

impl TypeError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeError {}

type TaggedTypeTable = HashMap<(TagType, String), TaggedTypeDecl>;

/// Type-check a program. Returns `Ok(CheckedProgram)` if well-typed, `Err(TypeError)` on first error.
/// Requires a `main` function with signature `void main()`.
pub fn type_check(program: &UncheckedProgram) -> Result<CheckedProgram, TypeError> {
    let tagged_table = build_tagged_type_table(&program.tagged_types)?;

    let main_fn = program.functions.iter().find(|f| f.name == "main");
    match main_fn {
        None => return Err(TypeError::new("program must have a main function")),
        Some(f) => {
            if f.return_type != Type::Unit {
                return Err(TypeError::new("main function must return void"));
            }
            if !f.params.is_empty() {
                return Err(TypeError::new("main function must have no parameters"));
            }
        }
    }

    let mut env = Environment::<Type>::new();

    // Register native stdlib functions as Type::Function bindings.
    let registry = NativeRegistry::default();
    for (name, entry) in registry.iter() {
        env.declare(
            name.clone(),
            Type::Function {
                params: entry.params.clone(),
                return_type: Box::new(entry.return_type.clone()),
            },
        );
    }

    // Register user-defined function signatures as Type::Function bindings.
    for f in &program.functions {
        let param_tys = f.params.iter().map(|param| param.ty.clone()).collect();
        env.declare(
            f.name.clone(),
            Type::Function {
                params: param_tys,
                return_type: Box::new(f.return_type.clone()),
            },
        );
    }

    // Clean snapshot: only function bindings, no variable bindings.
    let fn_snapshot = env.snapshot();

    let mut functions = Vec::new();
    for f in &program.functions {
        let checked = type_check_fun_decl(f, &mut env, &fn_snapshot, &tagged_table)?;
        functions.push(checked);
    }
    Ok(Program {
        tagged_types: program.tagged_types.clone(),
        functions,
    })
}

fn type_check_fun_decl(
    f: &UncheckedFunDecl,
    env: &mut Environment<Type>,
    fn_snapshot: &HashMap<String, Type>,
    tagged_table: &TaggedTypeTable,
) -> Result<CheckedFunDecl, TypeError> {
    // Restore to clean function-only state, then add parameters.
    env.restore(fn_snapshot.clone());
    for param in &f.params {
        env.declare(param.name.clone(), param.ty.clone());
    }
    let body = type_check_stmt(&f.body, env, &f.return_type, tagged_table)?;
    Ok(FunDecl {
        name: f.name.clone(),
        params: f.params.clone(),
        return_type: f.return_type.clone(),
        body: Box::new(body),
    })
}

fn type_check_stmt(
    s: &UncheckedStmt,
    env: &mut Environment<Type>,
    expected_return: &Type,
    tagged_table: &TaggedTypeTable,
) -> Result<CheckedStmt, TypeError> {
    let stmt = match &s.stmt {
        Statement::Decl { name, ty, init } => {
            if ty == &Type::Unit {
                return Err(TypeError::new("cannot declare variable of type void"));
            }
            if let Type::Tagged { tag_type, tag_name } = ty {
                if !tagged_table.contains_key(&(tag_type.clone(), tag_name.clone())) {
                    return Err(TypeError::new(format!(
                        "unknown tagged type: {:?} {}",
                        tag_type, tag_name
                    )));
                }
            }
            if env.get(name).is_some() {
                return Err(TypeError::new(format!(
                    "redeclaration of variable: {}",
                    name
                )));
            }
            let init_checked = type_check_expr_to_typed(init, env, tagged_table)?;
            if matches!(ty, Type::Tagged { .. }) {
                if init_checked.ty != Type::Int {
                    return Err(TypeError::new(
                        "tagged-typed variable declarations currently require integer placeholder initializer",
                    ));
                }
            } else if !types_compatible(&init_checked.ty, ty) {
                return Err(TypeError::new(format!(
                    "declaration of {}: expected {:?}, got {:?}",
                    name, ty, init_checked.ty
                )));
            }
            env.declare(name.clone(), ty.clone());
            Statement::Decl {
                name: name.clone(),
                ty: ty.clone(),
                init: Box::new(init_checked),
            }
        }
        Statement::Assign { target, value } => {
            let value_checked = type_check_expr_to_typed(value, env, tagged_table)?;
            type_check_assign_target(&target.exp, &value_checked.ty, env, tagged_table)?;
            Statement::Assign {
                target: Box::new(type_check_expr_to_typed(target, env, tagged_table)?),
                value: Box::new(value_checked),
            }
        }
        Statement::Block { seq } => {
            let snapshot = env.snapshot();
            let mut checked = Vec::new();
            for st in seq {
                checked.push(type_check_stmt(st, env, expected_return, tagged_table)?);
            }
            env.restore(snapshot);
            Statement::Block { seq: checked }
        }
        Statement::Call { name, args } => {
            let args_checked: Result<Vec<_>, _> = args
                .iter()
                .map(|a| type_check_expr_to_typed(a, env, tagged_table))
                .collect();
            let args_checked = args_checked?;
            check_call(name, &args_checked, env)?;
            Statement::Call {
                name: name.clone(),
                args: args_checked,
            }
        }
        Statement::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_checked = type_check_expr_to_typed(cond, env, tagged_table)?;
            if cond_checked.ty != Type::Bool {
                return Err(TypeError::new(format!(
                    "if condition must be Bool, got {:?}",
                    cond_checked.ty
                )));
            }
            let then_checked = type_check_stmt(then_branch, env, expected_return, tagged_table)?;
            let else_checked = else_branch
                .as_ref()
                .map(|e| type_check_stmt(e, env, expected_return, tagged_table))
                .transpose()?;
            Statement::If {
                cond: Box::new(cond_checked),
                then_branch: Box::new(then_checked),
                else_branch: else_checked.map(Box::new),
            }
        }
        Statement::While { cond, body } => {
            let cond_checked = type_check_expr_to_typed(cond, env, tagged_table)?;
            if cond_checked.ty != Type::Bool {
                return Err(TypeError::new(format!(
                    "while condition must be Bool, got {:?}",
                    cond_checked.ty
                )));
            }
            let body_checked = type_check_stmt(body, env, expected_return, tagged_table)?;
            Statement::While {
                cond: Box::new(cond_checked),
                body: Box::new(body_checked),
            }
        }
        Statement::Return(expr) => match expr {
            None => {
                if *expected_return != Type::Unit {
                    return Err(TypeError::new(format!(
                        "non-void function must return a value of type {:?}",
                        expected_return
                    )));
                }
                Statement::Return(None)
            }
            Some(e) => {
                if *expected_return == Type::Unit {
                    return Err(TypeError::new("void function must not return a value"));
                }
                let checked = type_check_expr_to_typed(e, env, tagged_table)?;
                if !types_compatible(&checked.ty, expected_return) {
                    return Err(TypeError::new(format!(
                        "return type mismatch: expected {:?}, got {:?}",
                        expected_return, checked.ty
                    )));
                }
                Statement::Return(Some(Box::new(checked)))
            }
        },
    };
    Ok(StatementD {
        stmt,
        ty: Type::Unit,
    })
}

fn check_call(name: &str, args: &[CheckedExpr], env: &Environment<Type>) -> Result<(), TypeError> {
    match env.get(name) {
        Some(Type::Function {
            params: param_tys, ..
        }) => {
            if args.len() != param_tys.len() {
                return Err(TypeError::new(format!(
                    "function '{}' expects {} arguments, got {}",
                    name,
                    param_tys.len(),
                    args.len()
                )));
            }
            for (i, (arg, param_ty)) in args.iter().zip(param_tys.iter()).enumerate() {
                if !types_compatible(&arg.ty, param_ty) {
                    return Err(TypeError::new(format!(
                        "argument {} to {}: expected {:?}, got {:?}",
                        i + 1,
                        name,
                        param_ty,
                        arg.ty
                    )));
                }
            }
            Ok(())
        }
        Some(_) => Err(TypeError::new(format!("'{}' is not a function", name))),
        None => Err(TypeError::new(format!("undefined function: {}", name))),
    }
}

fn type_check_assign_target(
    target: &Expr<()>,
    value_ty: &Type,
    env: &Environment<Type>,
    tagged_table: &TaggedTypeTable,
) -> Result<(), TypeError> {
    match target {
        Expr::Ident(name) => {
            let declared_ty = env
                .get(name)
                .ok_or_else(|| TypeError::new(format!("undeclared variable: {}", name)))?;
            if !types_compatible(value_ty, declared_ty) {
                return Err(TypeError::new(format!(
                    "assignment to {}: expected {:?}, got {:?}",
                    name, declared_ty, value_ty
                )));
            }
            Ok(())
        }
        Expr::Index { base, index } => {
            let index_ty = type_check_expr(index, env, tagged_table)?;
            if index_ty != Type::Int {
                return Err(TypeError::new("array index must be Int"));
            }
            let base_ty = type_check_expr(base, env, tagged_table)?;
            if let Type::Array(elem) = &base_ty {
                if **elem != *value_ty {
                    return Err(TypeError::new("assignment type mismatch"));
                }
            } else {
                return Err(TypeError::new("indexed target must be array"));
            }
            Ok(())
        }
        Expr::Member { base, member } => {
            let base_ty = type_check_expr(base, env, tagged_table)?;
            match base_ty {
                Type::Tagged { tag_type, tag_name } => {
                    let decl = tagged_table
                        .get(&(tag_type.clone(), tag_name.clone()))
                        .ok_or_else(|| {
                            TypeError::new(format!(
                                "unknown tagged type in member assignment: {:?} {}",
                                tag_type, tag_name
                            ))
                        })?;

                    match tag_type {
                        TagType::Struct | TagType::Union => {
                            let field_ty = decl
                                .members
                                .iter()
                                .find_map(|m| match m {
                                    Member::Field(decl) if decl.name == *member => {
                                        Some(decl.ty.clone())
                                    }
                                    _ => None,
                                })
                                .ok_or_else(|| {
                                    TypeError::new(format!(
                                        "unknown member '{}' on {:?} {}",
                                        member, tag_type, tag_name
                                    ))
                                })?;

                            if !types_compatible(value_ty, &field_ty) {
                                return Err(TypeError::new(format!(
                                    "assignment to {}.{}: expected {:?}, got {:?}",
                                    tag_name, member, field_ty, value_ty
                                )));
                            }
                            Ok(())
                        }
                        TagType::Enum => Err(TypeError::new("cannot assign to enum members")),
                    }
                }
                other => Err(TypeError::new(format!(
                    "member assignment requires tagged base type, got {:?}",
                    other
                ))),
            }
        }
        _ => Err(TypeError::new("invalid assignment target")),
    }
}

fn type_check_expr_to_typed(
    e: &UncheckedExpr,
    env: &Environment<Type>,
    tagged_table: &TaggedTypeTable,
) -> Result<CheckedExpr, TypeError> {
    let ty = type_check_expr(e, env, tagged_table)?;
    let exp = type_check_expr_inner(&e.exp, env, tagged_table)?;
    Ok(ExprD { exp, ty })
}

fn type_check_expr_inner(
    e: &Expr<()>,
    env: &Environment<Type>,
    tagged_table: &TaggedTypeTable,
) -> Result<Expr<Type>, TypeError> {
    match e {
        Expr::Literal(l) => Ok(Expr::Literal(l.clone())),
        Expr::Ident(name) => Ok(Expr::Ident(name.clone())),
        Expr::Neg(inner) => Ok(Expr::Neg(Box::new(type_check_expr_to_typed(
            inner,
            env,
            tagged_table,
        )?))),
        Expr::Add(l, r) => Ok(Expr::Add(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Sub(l, r) => Ok(Expr::Sub(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Mul(l, r) => Ok(Expr::Mul(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Div(l, r) => Ok(Expr::Div(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Eq(l, r) => Ok(Expr::Eq(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Ne(l, r) => Ok(Expr::Ne(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Lt(l, r) => Ok(Expr::Lt(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Le(l, r) => Ok(Expr::Le(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Gt(l, r) => Ok(Expr::Gt(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Ge(l, r) => Ok(Expr::Ge(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Not(inner) => Ok(Expr::Not(Box::new(type_check_expr_to_typed(
            inner,
            env,
            tagged_table,
        )?))),
        Expr::And(l, r) => Ok(Expr::And(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Or(l, r) => Ok(Expr::Or(
            Box::new(type_check_expr_to_typed(l, env, tagged_table)?),
            Box::new(type_check_expr_to_typed(r, env, tagged_table)?),
        )),
        Expr::Call { name, args } => {
            let args_checked: Result<Vec<_>, _> = args
                .iter()
                .map(|a| type_check_expr_to_typed(a, env, tagged_table))
                .collect();
            Ok(Expr::Call {
                name: name.clone(),
                args: args_checked?,
            })
        }
        Expr::ArrayLit(elems) => {
            let elems_checked: Result<Vec<_>, _> = elems
                .iter()
                .map(|e| type_check_expr_to_typed(e, env, tagged_table))
                .collect();
            Ok(Expr::ArrayLit(elems_checked?))
        }
        Expr::Index { base, index } => Ok(Expr::Index {
            base: Box::new(type_check_expr_to_typed(base, env, tagged_table)?),
            index: Box::new(type_check_expr_to_typed(index, env, tagged_table)?),
        }),
        Expr::Member { base, member } => Ok(Expr::Member {
            base: Box::new(type_check_expr_to_typed(base, env, tagged_table)?),
            member: member.clone(),
        }),
    }
}

fn type_check_expr(
    e: &UncheckedExpr,
    env: &Environment<Type>,
    tagged_table: &TaggedTypeTable,
) -> Result<Type, TypeError> {
    match &e.exp {
        Expr::Literal(l) => Ok(literal_type(l)),
        Expr::Ident(name) => match env.get(name) {
            Some(Type::Function { .. }) => Err(TypeError::new(format!(
                "cannot use function '{}' as a value",
                name
            ))),
            Some(ty) => Ok(ty.clone()),
            None => Err(TypeError::new(format!("undeclared variable: {}", name))),
        },
        Expr::Neg(inner) => {
            let ty = type_check_expr(inner, env, tagged_table)?;
            if matches!(ty, Type::Int | Type::Float) {
                Ok(ty)
            } else {
                Err(TypeError::new("unary minus requires Int or Float"))
            }
        }
        Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) => {
            let lt = type_check_expr(l, env, tagged_table)?;
            let rt = type_check_expr(r, env, tagged_table)?;
            numeric_binop_result(&lt, &rt)
        }
        Expr::Eq(l, r) | Expr::Ne(l, r) => {
            let lt = type_check_expr(l, env, tagged_table)?;
            let rt = type_check_expr(r, env, tagged_table)?;
            if !types_compatible(&lt, &rt) {
                return Err(TypeError::new(format!(
                    "equality operands must have compatible types, got {:?} and {:?}",
                    lt, rt
                )));
            }
            Ok(Type::Bool)
        }
        Expr::Lt(l, r) | Expr::Le(l, r) | Expr::Gt(l, r) | Expr::Ge(l, r) => {
            let lt = type_check_expr(l, env, tagged_table)?;
            let rt = type_check_expr(r, env, tagged_table)?;
            if !is_numeric(&lt) || !is_numeric(&rt) {
                return Err(TypeError::new(format!(
                    "ordering comparison requires numeric operands, got {:?} and {:?}",
                    lt, rt
                )));
            }
            Ok(Type::Bool)
        }
        Expr::Not(inner) => {
            let ty = type_check_expr(inner, env, tagged_table)?;
            if ty == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(TypeError::new("not requires Bool operand"))
            }
        }
        Expr::And(l, r) | Expr::Or(l, r) => {
            let lt = type_check_expr(l, env, tagged_table)?;
            let rt = type_check_expr(r, env, tagged_table)?;
            if lt == Type::Bool && rt == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(TypeError::new("and/or require Bool operands"))
            }
        }
        Expr::Call { name, args } => {
            let args_checked: Result<Vec<_>, _> = args
                .iter()
                .map(|a| type_check_expr_to_typed(a, env, tagged_table))
                .collect();
            let args_checked = args_checked?;
            match env.get(name) {
                Some(Type::Function {
                    params: param_tys,
                    return_type,
                }) => {
                    if args_checked.len() != param_tys.len() {
                        return Err(TypeError::new(format!(
                            "function '{}' expects {} arguments, got {}",
                            name,
                            param_tys.len(),
                            args_checked.len()
                        )));
                    }
                    for (i, (arg, param_ty)) in
                        args_checked.iter().zip(param_tys.iter()).enumerate()
                    {
                        if !types_compatible(&arg.ty, param_ty) {
                            return Err(TypeError::new(format!(
                                "argument {} to {}: expected {:?}, got {:?}",
                                i + 1,
                                name,
                                param_ty,
                                arg.ty
                            )));
                        }
                    }
                    Ok((**return_type).clone())
                }
                Some(_) => Err(TypeError::new(format!("'{}' is not a function", name))),
                None => Err(TypeError::new(format!("undefined function: {}", name))),
            }
        }
        Expr::ArrayLit(elems) => {
            if elems.is_empty() {
                return Err(TypeError::new("empty array literal needs type annotation"));
            }
            let first = type_check_expr(&elems[0], env, tagged_table)?;
            for e in elems.iter().skip(1) {
                let ty = type_check_expr(e, env, tagged_table)?;
                if !types_compatible(&first, &ty) {
                    return Err(TypeError::new("array elements must have same type"));
                }
            }
            Ok(Type::Array(Box::new(first)))
        }
        Expr::Index { base, index } => {
            let index_ty = type_check_expr(index, env, tagged_table)?;
            if index_ty != Type::Int {
                return Err(TypeError::new("array index must be Int"));
            }
            let base_ty = type_check_expr(base, env, tagged_table)?;
            if let Type::Array(elem) = base_ty {
                Ok(*elem)
            } else {
                Err(TypeError::new("indexed expression must be array"))
            }
        }
        Expr::Member { base, member } => {
            let base_ty = type_check_expr(base, env, tagged_table)?;
            match base_ty {
                Type::Tagged { tag_type, tag_name } => {
                    let decl = tagged_table
                        .get(&(tag_type.clone(), tag_name.clone()))
                        .ok_or_else(|| {
                            TypeError::new(format!(
                                "unknown tagged type in member access: {:?} {}",
                                tag_type, tag_name
                            ))
                        })?;

                    match tag_type {
                        TagType::Struct | TagType::Union => decl
                            .members
                            .iter()
                            .find_map(|m| match m {
                                Member::Field(decl) if decl.name == *member => {
                                    Some(decl.ty.clone())
                                }
                                _ => None,
                            })
                            .ok_or_else(|| {
                                TypeError::new(format!(
                                    "unknown member '{}' on {:?} {}",
                                    member, tag_type, tag_name
                                ))
                            }),
                        TagType::Enum => {
                            let exists = decl.members.iter().any(|m| match m {
                                Member::Enumerator { name, .. } => name == member,
                                _ => false,
                            });
                            if exists {
                                Ok(Type::Int)
                            } else {
                                Err(TypeError::new(format!(
                                    "unknown enumerator '{}' on enum {}",
                                    member, tag_name
                                )))
                            }
                        }
                    }
                }
                other => Err(TypeError::new(format!(
                    "member access requires tagged base type, got {:?}",
                    other
                ))),
            }
        }
    }
}

fn literal_type(l: &Literal) -> Type {
    match l {
        Literal::Int(_) => Type::Int,
        Literal::Float(_) => Type::Float,
        Literal::Str(_) => Type::Str,
        Literal::Bool(_) => Type::Bool,
    }
}

fn numeric_binop_result(l: &Type, r: &Type) -> Result<Type, TypeError> {
    match (l, r) {
        (Type::Int, Type::Int) => Ok(Type::Int),
        (Type::Int, Type::Float) | (Type::Float, Type::Int) | (Type::Float, Type::Float) => {
            Ok(Type::Float)
        }
        _ => Err(TypeError::new("arithmetic operands must be Int or Float")),
    }
}

fn is_numeric(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::Float)
}

fn types_compatible(a: &Type, b: &Type) -> bool {
    match (a, b) {
        // Any parameter accepts any argument type.
        (_, Type::Any) => true,
        (Type::Int, Type::Int)
        | (Type::Float, Type::Float)
        | (Type::Bool, Type::Bool)
        | (Type::Str, Type::Str)
        | (Type::Unit, Type::Unit) => true,
        (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
        (Type::Array(a), Type::Array(b)) => types_compatible(a, b),
        (
            Type::Tagged {
                tag_type: a_kind,
                tag_name: a_name,
            },
            Type::Tagged {
                tag_type: b_kind,
                tag_name: b_name,
            },
        ) => a_kind == b_kind && a_name == b_name,
        _ => false,
    }
}

fn build_tagged_type_table(tagged_types: &[TaggedTypeDecl]) -> Result<TaggedTypeTable, TypeError> {
    let mut seen = std::collections::HashSet::<(TagType, String)>::new();
    let mut table = TaggedTypeTable::new();

    for decl in tagged_types {
        let key = (decl.tag_type.clone(), decl.tag_name.clone());

        if !seen.insert(key.clone()) {
            return Err(TypeError::new(format!(
                "duplicate tagged type declaration: {:?} {}",
                decl.tag_type, decl.tag_name
            )));
        }

        if decl.members.is_empty() {
            return Err(TypeError::new(format!(
                "tagged type declaration '{}' must declare at least one member",
                decl.tag_name
            )));
        }

        table.insert(key, decl.clone());
    }

    Ok(table)
}
