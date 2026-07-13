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

use std::collections::{HashMap, HashSet};

use crate::environment::{build_type_decl_map, Environment};
use crate::ir::ast::{
    CheckedExpr, CheckedFunDecl, CheckedProgram, CheckedStmt, Expr, ExprD, FunDecl, Literal,
    MatchArm, Program, Statement, StatementD, Type, UncheckedExpr, UncheckedFunDecl,
    UncheckedProgram, UncheckedStmt, UDTDecl, UDTKind, UDTMember,
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

fn check_type_decls_unique(decls: &[UDTDecl]) -> Result<(), TypeError> {
    let mut seen = HashSet::new();
    for decl in decls {
        let key = (decl.specifier.clone(), decl.identifier.clone());
        if !seen.insert(key) {
            return Err(TypeError::new(format!(
                "duplicate type declaration: {:?} {}",
                decl.specifier, decl.identifier
            )));
        }
    }
    Ok(())
}

fn validate_program_type_references(
    program: &UncheckedProgram,
    env: &Environment<Type>,
) -> Result<(), TypeError> {
    for decl in &program.type_declarations {
        for member in &decl.members {
            match member {
                UDTMember::Field(field) => validate_type_reference(
                    &field.ty,
                    env,
                    &format!("field '{}.{}'", decl.identifier, field.name),
                )?,
                UDTMember::EnumVariant { name, ty: Some(ty) } => validate_type_reference(
                    ty,
                    env,
                    &format!("payload for variant '{}.{}'", decl.identifier, name),
                )?,
                UDTMember::EnumVariant { ty: None, .. } => {}
            }
        }
    }

    for function in &program.functions {
        validate_type_reference(
            &function.return_type,
            env,
            &format!("return type of function '{}'", function.name),
        )?;
        for param in &function.params {
            validate_type_reference(
                &param.ty,
                env,
                &format!("parameter '{}' of function '{}'", param.name, function.name),
            )?;
        }
    }

    Ok(())
}

fn validate_type_reference(
    ty: &Type,
    env: &Environment<Type>,
    context: &str,
) -> Result<(), TypeError> {
    match ty {
        Type::Array(inner) => validate_type_reference(inner, env, context),
        Type::Struct(name) => {
            if env.has_type_decl(&UDTKind::Struct, name) {
                Ok(())
            } else {
                Err(TypeError::new(format!(
                    "unknown struct type '{}' in {}",
                    name, context
                )))
            }
        }
        Type::Enum(name) => {
            if env.has_type_decl(&UDTKind::Enum, name) {
                Ok(())
            } else {
                Err(TypeError::new(format!(
                    "unknown enum type '{}' in {}",
                    name, context
                )))
            }
        }
        Type::Function { params, return_type } => {
            for param in params {
                validate_type_reference(param, env, context)?;
            }
            validate_type_reference(return_type, env, context)
        }
        _ => Ok(()),
    }
}

/// Type-check a program. Returns `Ok(CheckedProgram)` if well-typed, `Err(TypeError)` on first error.
/// Requires a `main` function with signature `void main()`.
pub fn type_check(program: &UncheckedProgram) -> Result<CheckedProgram, TypeError> {
    check_type_decls_unique(&program.type_declarations)?;
    let type_map = build_type_decl_map(&program.type_declarations);
    let mut env = Environment::with_type_decls(type_map);

    validate_program_type_references(program, &env)?;

    let main_fn = program.main_function();
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
        let checked = type_check_fun_decl(f, &mut env, &fn_snapshot)?;
        functions.push(checked);
    }
    Ok(Program {
        type_declarations: program.type_declarations.clone(),
        functions,
    })
}

fn type_check_fun_decl(
    f: &UncheckedFunDecl,
    env: &mut Environment<Type>,
    fn_snapshot: &HashMap<String, Type>,
) -> Result<CheckedFunDecl, TypeError> {
    // Restore to clean function-only state, then add parameters.
    env.restore(fn_snapshot.clone());
    for param in &f.params {
        env.declare(param.name.clone(), param.ty.clone());
    }
    let body = type_check_stmt(&f.body, env, &f.return_type)?;
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
) -> Result<CheckedStmt, TypeError> {
    let stmt = match &s.stmt {
        Statement::Decl { name, ty, init } => {
            if ty == &Type::Unit {
                return Err(TypeError::new("cannot declare variable of type void"));
            }
            validate_type_reference(ty, env, &format!("declaration of variable '{}'", name))?;
            if env.get(name).is_some() {
                return Err(TypeError::new(format!(
                    "redeclaration of variable: {}",
                    name
                )));
            }
            let init_checked = type_check_expr(init, env, Some(ty))?;
            if !types_compatible(&init_checked.ty, ty) {
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
            let target_checked = type_check_expr(target, env, None)?;
            if !is_assignable_target(&target.exp) {
                return Err(TypeError::new("invalid assignment target"));
            }
            let value_checked = type_check_expr(value, env, Some(&target_checked.ty))?;
            if !types_compatible(&value_checked.ty, &target_checked.ty) {
                return Err(TypeError::new(format!(
                    "assignment: expected {:?}, got {:?}",
                    target_checked.ty, value_checked.ty
                )));
            }
            Statement::Assign {
                target: Box::new(target_checked),
                value: Box::new(value_checked),
            }
        }
        Statement::Block { seq } => {
            let snapshot = env.snapshot();
            let mut checked = Vec::new();
            for st in seq {
                checked.push(type_check_stmt(st, env, expected_return)?);
            }
            env.restore(snapshot);
            Statement::Block { seq: checked }
        }
        Statement::Call { name, args } => {
            let params = match env.get(name) {
                Some(Type::Function { params, .. }) => params,
                Some(_) => {
                    return Err(TypeError::new(format!(
                        "'{}' is not a function",
                        name
                    )))
                }
                None => {
                    return Err(TypeError::new(format!(
                        "undefined function: {}",
                        name
                    )))
                }
            };
            if args.len() != params.len() {
                return Err(TypeError::new(format!(
                    "function '{}' expects {} arguments, got {}",
                    name,
                    params.len(),
                    args.len()
                )));
            }
            let args_checked: Result<Vec<_>, _> = args
                .iter()
                .zip(params.iter())
                .map(|(a, p)| type_check_expr(a, env, Some(p)))
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
            let cond_checked = type_check_expr(cond, env, None)?;
            if cond_checked.ty != Type::Bool {
                return Err(TypeError::new(format!(
                    "if condition must be Bool, got {:?}",
                    cond_checked.ty
                )));
            }
            let then_checked = type_check_stmt(then_branch, env, expected_return)?;
            let else_checked = else_branch
                .as_ref()
                .map(|e| type_check_stmt(e, env, expected_return))
                .transpose()?;
            Statement::If {
                cond: Box::new(cond_checked),
                then_branch: Box::new(then_checked),
                else_branch: else_checked.map(Box::new),
            }
        }
        Statement::While { cond, body } => {
            let cond_checked = type_check_expr(cond, env, None)?;
            if cond_checked.ty != Type::Bool {
                return Err(TypeError::new(format!(
                    "while condition must be Bool, got {:?}",
                    cond_checked.ty
                )));
            }
            let body_checked = type_check_stmt(body, env, expected_return)?;
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
                let checked = type_check_expr(e, env, Some(expected_return))?;
                if !types_compatible(&checked.ty, expected_return) {
                    return Err(TypeError::new(format!(
                        "return type mismatch: expected {:?}, got {:?}",
                        expected_return, checked.ty
                    )));
                }
                Statement::Return(Some(Box::new(checked)))
            }
        }
        Statement::Match { target, arms } => {
            let target_checked = type_check_expr(target, env, None)?;
            let enum_name = match &target_checked.ty {
                Type::Enum(name) => name.clone(),
                other => {
                    return Err(TypeError::new(format!(
                        "match target must be an enum, got {:?}",
                        other
                    )))
                }
            };
            let decl = env
                .get_type_decl(&UDTKind::Enum, &enum_name)
                .ok_or_else(|| {
                    TypeError::new(format!("unknown enum type in match: {}", enum_name))
                })?
                .clone();
            let expected_variants: HashSet<String> = decl
                .members
                .iter()
                .filter_map(|m| match m {
                    UDTMember::EnumVariant { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect();
            let mut seen_variants = HashSet::new();
            let mut checked_arms = Vec::new();
            for arm in arms {
                if !seen_variants.insert(arm.variant.clone()) {
                    return Err(TypeError::new(format!(
                        "duplicate match arm for variant '{}' in enum {}",
                        arm.variant, enum_name
                    )));
                }
                let payload_ty = decl.members.iter().find_map(|m| match m {
                    UDTMember::EnumVariant { name, ty }
                        if *name == arm.variant =>
                    {
                        Some(ty.clone())
                    }
                    _ => None,
                })
                .ok_or_else(|| {
                    TypeError::new(format!(
                        "unknown variant '{}' for enum {}",
                        arm.variant, enum_name
                    ))
                })?;
                let snapshot = env.snapshot();
                if let Some(ref pty) = payload_ty {
                    env.declare(arm.variant.clone(), pty.clone());
                }
                let body_checked = type_check_stmt(&arm.body, env, expected_return)?;
                env.restore(snapshot);
                checked_arms.push(MatchArm {
                    variant: arm.variant.clone(),
                    binding: payload_ty.as_ref().map(|_| arm.variant.clone()),
                    body: Box::new(body_checked),
                });
            }
            let mut missing_variants = expected_variants
                .difference(&seen_variants)
                .cloned()
                .collect::<Vec<_>>();
            missing_variants.sort();
            if !missing_variants.is_empty() {
                return Err(TypeError::new(format!(
                    "non-exhaustive match for enum {}: missing variants {:?}",
                    enum_name, missing_variants
                )));
            }
            Statement::Match {
                target: Box::new(target_checked),
                arms: checked_arms,
            }
        }
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

fn is_assignable_target(target: &Expr<()>) -> bool {
    match target {
        Expr::Ident(_) => true,
        Expr::Index { base, .. } | Expr::Member { base, .. } => is_assignable_target(&base.exp),
        _ => false,
    }
}

/// Unified expression type checker.
///
/// Checks an expression and returns a [`CheckedExpr`] with the inferred type.
/// When `expected` is `Some(Struct/Enum(..))` and the expression is an
/// `Expr::Init`, it dispatches to the appropriate struct/enum init checker.
/// For all other expressions, `expected` is unused (sub-expressions infer
/// their own types).
fn type_check_expr(
    e: &UncheckedExpr,
    env: &Environment<Type>,
    expected: Option<&Type>,
) -> Result<CheckedExpr, TypeError> {
    let result = match &e.exp {
        Expr::Literal(l) => ExprD {
            exp: Expr::Literal(l.clone()),
            ty: literal_type(l),
        },
        Expr::Ident(name) => {
            let ty = match env.get(name) {
                Some(Type::Function { .. }) => {
                    return Err(TypeError::new(format!(
                        "cannot use function '{}' as a value",
                        name
                    )))
                }
                Some(ty) => ty.clone(),
                None => {
                    return Err(TypeError::new(format!(
                        "undeclared variable: {}",
                        name
                    )))
                }
            };
            ExprD {
                exp: Expr::Ident(name.clone()),
                ty,
            }
        }
        Expr::Neg(inner) => {
            let inner = type_check_expr(inner, env, None)?;
            if !matches!(inner.ty, Type::Int | Type::Float) {
                return Err(TypeError::new("unary minus requires Int or Float"));
            }
            let ty = inner.ty.clone();
            ExprD {
                exp: Expr::Neg(Box::new(inner)),
                ty,
            }
        }
        Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) => {
            let l = type_check_expr(l, env, None)?;
            let r = type_check_expr(r, env, None)?;
            let ty = numeric_binop_result(&l.ty, &r.ty)?;
            let op = match &e.exp {
                Expr::Add(_, _) => Expr::Add(Box::new(l), Box::new(r)),
                Expr::Sub(_, _) => Expr::Sub(Box::new(l), Box::new(r)),
                Expr::Mul(_, _) => Expr::Mul(Box::new(l), Box::new(r)),
                Expr::Div(_, _) => Expr::Div(Box::new(l), Box::new(r)),
                _ => unreachable!(),
            };
            ExprD { exp: op, ty }
        }
        Expr::Eq(l, r) | Expr::Ne(l, r) => {
            let l = type_check_expr(l, env, None)?;
            let r = type_check_expr(r, env, None)?;
            if !types_compatible(&l.ty, &r.ty) {
                return Err(TypeError::new(format!(
                    "equality operands must have compatible types, got {:?} and {:?}",
                    l.ty, r.ty
                )));
            }
            let op = match &e.exp {
                Expr::Eq(_, _) => Expr::Eq(Box::new(l), Box::new(r)),
                Expr::Ne(_, _) => Expr::Ne(Box::new(l), Box::new(r)),
                _ => unreachable!(),
            };
            ExprD {
                exp: op,
                ty: Type::Bool,
            }
        }
        Expr::Lt(l, r) | Expr::Le(l, r) | Expr::Gt(l, r) | Expr::Ge(l, r) => {
            let l = type_check_expr(l, env, None)?;
            let r = type_check_expr(r, env, None)?;
            if !is_numeric(&l.ty) || !is_numeric(&r.ty) {
                return Err(TypeError::new(format!(
                    "ordering comparison requires numeric operands, got {:?} and {:?}",
                    l.ty, r.ty
                )));
            }
            let op = match &e.exp {
                Expr::Lt(_, _) => Expr::Lt(Box::new(l), Box::new(r)),
                Expr::Le(_, _) => Expr::Le(Box::new(l), Box::new(r)),
                Expr::Gt(_, _) => Expr::Gt(Box::new(l), Box::new(r)),
                Expr::Ge(_, _) => Expr::Ge(Box::new(l), Box::new(r)),
                _ => unreachable!(),
            };
            ExprD {
                exp: op,
                ty: Type::Bool,
            }
        }
        Expr::Not(inner) => {
            let inner = type_check_expr(inner, env, None)?;
            if inner.ty != Type::Bool {
                return Err(TypeError::new("not requires Bool operand"));
            }
            ExprD {
                exp: Expr::Not(Box::new(inner)),
                ty: Type::Bool,
            }
        }
        Expr::And(l, r) | Expr::Or(l, r) => {
            let l = type_check_expr(l, env, None)?;
            let r = type_check_expr(r, env, None)?;
            if l.ty != Type::Bool || r.ty != Type::Bool {
                return Err(TypeError::new("and/or require Bool operands"));
            }
            let op = match &e.exp {
                Expr::And(_, _) => Expr::And(Box::new(l), Box::new(r)),
                Expr::Or(_, _) => Expr::Or(Box::new(l), Box::new(r)),
                _ => unreachable!(),
            };
            ExprD {
                exp: op,
                ty: Type::Bool,
            }
        }
        Expr::Call { name, args } => {
            let (params, return_type) = match env.get(name) {
                Some(Type::Function { params, return_type }) => (params, return_type),
                Some(_) => {
                    return Err(TypeError::new(format!(
                        "'{}' is not a function",
                        name
                    )))
                }
                None => {
                    return Err(TypeError::new(format!(
                        "undefined function: {}",
                        name
                    )))
                }
            };
            if args.len() != params.len() {
                return Err(TypeError::new(format!(
                    "function '{}' expects {} arguments, got {}",
                    name,
                    params.len(),
                    args.len()
                )));
            }
            let args_checked: Result<Vec<_>, _> = args
                .iter()
                .zip(params.iter())
                .map(|(a, p)| type_check_expr(a, env, Some(p)))
                .collect();
            let args_checked = args_checked?;
            for (i, (arg, param_ty)) in
                args_checked.iter().zip(params.iter()).enumerate()
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
            ExprD {
                exp: Expr::Call {
                    name: name.clone(),
                    args: args_checked,
                },
                ty: (**return_type).clone(),
            }
        }
        Expr::ArrayLit(elems) => {
            if elems.is_empty() {
                return Err(TypeError::new(
                    "empty array literal needs type annotation",
                ));
            }
            let elems_checked: Result<Vec<_>, _> = elems
                .iter()
                .map(|e| type_check_expr(e, env, None))
                .collect();
            let elems_checked = elems_checked?;
            let first_ty = elems_checked[0].ty.clone();
            for e in elems_checked.iter().skip(1) {
                if !types_compatible(&e.ty, &first_ty) {
                    return Err(TypeError::new(
                        "array elements must have same type",
                    ));
                }
            }
            ExprD {
                exp: Expr::ArrayLit(elems_checked),
                ty: Type::Array(Box::new(first_ty)),
            }
        }
        Expr::Index { base, index } => {
            let index = type_check_expr(index, env, None)?;
            if index.ty != Type::Int {
                return Err(TypeError::new("array index must be Int"));
            }
            let base = type_check_expr(base, env, None)?;
            let elem_ty = match &base.ty {
                Type::Array(elem) => (**elem).clone(),
                _ => {
                    return Err(TypeError::new("indexed expression must be array"))
                }
            };
            ExprD {
                exp: Expr::Index {
                    base: Box::new(base),
                    index: Box::new(index),
                },
                ty: elem_ty,
            }
        }
        Expr::Member { base, member } => {
            let base = type_check_expr(base, env, None)?;
            let field_ty = match &base.ty {
                Type::Struct(ref identifier) => {
                    let decl = env
                        .get_type_decl(&UDTKind::Struct, identifier)
                        .ok_or_else(|| {
                            TypeError::new(format!(
                                "unknown struct type in member access: {}",
                                identifier
                            ))
                        })?;
                    decl.members
                        .iter()
                        .find_map(|m| match m {
                            UDTMember::Field(decl) if decl.name == *member => {
                                Some(decl.ty.clone())
                            }
                            _ => None,
                        })
                        .ok_or_else(|| {
                            TypeError::new(format!(
                                "unknown member '{}' on struct {}",
                                member, identifier
                            ))
                        })?
                }
                Type::Enum(ref identifier) => {
                    return Err(TypeError::new(format!(
                        "cannot access enum variants directly, use match for '{}'",
                        identifier
                    )))
                }
                other => {
                    return Err(TypeError::new(format!(
                        "member access requires struct base type, got {:?}",
                        other
                    )))
                }
            };
            ExprD {
                exp: Expr::Member {
                    base: Box::new(base),
                    member: member.clone(),
                },
                ty: field_ty,
            }
        }
        Expr::Init { fields } => match expected {
            Some(Type::Struct(struct_name)) => {
                check_struct_init(fields, struct_name, env)?
            }
            Some(Type::Enum(enum_name)) => {
                check_enum_init(fields, enum_name, env)?
            }
            Some(other) => {
                return Err(TypeError::new(format!(
                    "struct/enum init used with non-struct, non-enum type {:?}",
                    other
                )))
            }
            None => {
                return Err(TypeError::new(
                    "struct/enum init used outside of variable declaration",
                ))
            }
        },
        Expr::Cast { ty, expr } => {
            validate_type_reference(ty, env, "cast target type")?;
            let checked_inner = type_check_expr(expr, env, Some(ty))?;
            if !is_valid_cast(&checked_inner.ty, ty, &checked_inner.exp) {
                return Err(TypeError::new(format!(
                    "invalid cast from {:?} to {:?}",
                    checked_inner.ty, ty
                )));
            }
            ExprD {
                exp: Expr::Cast {
                    ty: ty.clone(),
                    expr: Box::new(checked_inner),
                },
                ty: ty.clone(),
            }
        }
        Expr::EnumVariant {
            enum_name,
            variant,
            payload,
        } => {
            let checked_payload = match payload {
                Some(e) => Some(Box::new(type_check_expr(e, env, None)?)),
                None => None,
            };
            let ty = match enum_name {
                Some(name) => Type::Enum(name.clone()),
                None => {
                    return Err(TypeError::new(
                        "enum variant without enum type",
                    ))
                }
            };
            ExprD {
                exp: Expr::EnumVariant {
                    enum_name: enum_name.clone(),
                    variant: variant.clone(),
                    payload: checked_payload,
                },
                ty,
            }
        }
    };
    Ok(result)
}

fn check_struct_init(
    fields: &[(String, Option<UncheckedExpr>)],
    struct_name: &str,
    env: &Environment<Type>,
) -> Result<CheckedExpr, TypeError> {
    let decl = env
        .get_type_decl(&UDTKind::Struct, struct_name)
        .ok_or_else(|| TypeError::new(format!("unknown struct type: {}", struct_name)))?;

    let mut expected_fields: std::collections::HashSet<String> = decl
        .members
        .iter()
        .filter_map(|m| match m {
            UDTMember::Field(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect();

    let mut checked_fields = Vec::new();
    for (field_name, field_expr_opt) in fields {
        let field_expr = field_expr_opt.as_ref().ok_or_else(|| {
            TypeError::new(format!(
                "field '{}' in struct {} must have a value",
                field_name, struct_name
            ))
        })?;

        let field_decl = decl
            .members
            .iter()
            .find_map(|m| match m {
                UDTMember::Field(f) if f.name == *field_name => Some(f),
                _ => None,
            })
            .ok_or_else(|| {
                TypeError::new(format!(
                    "unknown field '{}' in struct {}",
                    field_name, struct_name
                ))
            })?;

        let checked = type_check_expr(field_expr, env, Some(&field_decl.ty))?;
        if !types_compatible(&checked.ty, &field_decl.ty) {
            return Err(TypeError::new(format!(
                "field '{}' expects {:?}, got {:?}",
                field_name, field_decl.ty, checked.ty
            )));
        }
        if !expected_fields.remove(field_name) {
            return Err(TypeError::new(format!(
                "duplicate field '{}' in struct {} initializer",
                field_name, struct_name
            )));
        }
        checked_fields.push((field_name.clone(), Some(checked)));
    }

    if !expected_fields.is_empty() {
        return Err(TypeError::new(format!(
            "missing fields in struct {} initializer: {:?}",
            struct_name,
            expected_fields.iter().collect::<Vec<_>>()
        )));
    }

    Ok(ExprD {
        exp: Expr::Init {
            fields: checked_fields,
        },
        ty: Type::Struct(struct_name.to_string()),
    })
}

fn check_enum_init(
    fields: &[(String, Option<UncheckedExpr>)],
    enum_name: &str,
    env: &Environment<Type>,
) -> Result<CheckedExpr, TypeError> {
    if fields.len() != 1 {
        return Err(TypeError::new(format!(
            "enum init requires exactly one variant, got {}",
            fields.len()
        )));
    }
    let (variant, payload_opt) = &fields[0];
    let decl = env
        .get_type_decl(&UDTKind::Enum, enum_name)
        .ok_or_else(|| TypeError::new(format!("unknown enum type: {}", enum_name)))?;
    let member = decl
        .members
        .iter()
        .find(|m| matches!(m, UDTMember::EnumVariant { name: n, .. } if n == variant))
        .ok_or_else(|| {
            TypeError::new(format!(
                "unknown variant '{}' for enum {}",
                variant, enum_name
            ))
        })?;
    match (member, payload_opt) {
        (UDTMember::EnumVariant { ty: Some(expected_ty), .. }, Some(payload_expr)) => {
            let checked = type_check_expr(payload_expr, env, Some(expected_ty))?;
            if !types_compatible(&checked.ty, expected_ty) {
                return Err(TypeError::new(format!(
                    "variant '{}' expects {:?} payload, got {:?}",
                    variant, expected_ty, checked.ty
                )));
            }
            Ok(ExprD {
                exp: Expr::EnumVariant {
                    enum_name: Some(enum_name.to_string()),
                    variant: variant.clone(),
                    payload: Some(Box::new(checked)),
                },
                ty: Type::Enum(enum_name.to_string()),
            })
        }
        (UDTMember::EnumVariant { ty: None, .. }, None) => Ok(ExprD {
            exp: Expr::EnumVariant {
                enum_name: Some(enum_name.to_string()),
                variant: variant.clone(),
                payload: None,
            },
            ty: Type::Enum(enum_name.to_string()),
        }),
        (UDTMember::EnumVariant { ty: Some(_), .. }, None) => {
            Err(TypeError::new(format!(
                "variant '{}' requires a payload value, use {{ .{} = expr }}",
                variant, variant
            )))
        }
        (UDTMember::EnumVariant { ty: None, .. }, Some(_)) => {
            Err(TypeError::new(format!(
                "variant '{}' is a unit variant and takes no value",
                variant
            )))
        }
        _ => unreachable!(),
    }
}

fn is_valid_cast(from: &Type, to: &Type, checked_expr: &Expr<Type>) -> bool {
    if from == to {
        return true;
    }

    if matches!((from, to), (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
        return true;
    }

    matches!(
        (to, checked_expr),
        (Type::Struct(_), Expr::Init { .. }) | (Type::Enum(_), Expr::EnumVariant { .. })
    )
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
        (Type::Struct(a), Type::Struct(b)) => a == b,
        (Type::Enum(a), Type::Enum(b)) => a == b,
        _ => false,
    }
}
