//! Unit tests for Project 7 — Constant Declarations.
//!
//! Tests are organised into three sections matching the implementation stages:
//!
//! 1. **Parser** — verifies that `const T name = expr` is parsed into a
//!    `Statement::ConstDecl` AST node (local and global scope).
//! 2. **Type Checker** — verifies that constants are accepted when used as
//!    values and rejected when used as assignment targets.
//! 3. **Interpreter** — verifies end-to-end execution: constants hold their
//!    initialiser values and global constants are visible in all functions.
//!
//! These tests are written *against the expected post-implementation API*.
//! They will all fail until the feature is implemented — that is intentional.
//! Run with `cargo test const_decl` to execute only this file.

use nom::combinator::all_consuming;

use mini_c::ir::ast::{Expr, Literal, Statement, StatementD, Type};
use mini_c::parser::{program, statement};
use mini_c::semantic::{type_check, TypeError};
use mini_c::{interpreter::interpret, parser::program as parse_program};

// ---------------------------------------------------------------------------
// Helpers (mirror the patterns used in type_checker.rs / interpreter.rs)
// ---------------------------------------------------------------------------

/// Parse a single MiniC statement from `src`.
fn parse_stmt(src: &str) -> Result<StatementD<()>, String> {
    all_consuming(statement)(src)
        .map(|(_, s)| s)
        .map_err(|e| format!("{:?}", e))
}

/// Parse a full MiniC program, type-check it, and return the result.
fn parse_and_type_check(src: &str) -> Result<mini_c::ir::ast::CheckedProgram, TypeError> {
    let (_, prog) = all_consuming(program)(src).map_err(|_| TypeError {
        message: "parse failed".to_string(),
    })?;
    type_check(&prog)
}

/// Parse, type-check, and interpret a full MiniC program.
fn run(src: &str) -> Result<(), String> {
    let unchecked = parse_program(src)
        .map_err(|e| format!("parse error: {:?}", e))
        .map(|(_, p)| p)?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
}

// ===========================================================================
// 1. PARSER TESTS
// ===========================================================================
//
// Strategy: call `statement` (or `all_consuming(program)` for global consts)
// and inspect the returned AST node.
// `Statement::ConstDecl` is the expected new variant.

// ---------------------------------------------------------------------------
// 1.1 Local const declarations (inside a function body)
// ---------------------------------------------------------------------------

/// `const int` declaration parses with the correct name, type, and literal.
#[test]
fn test_parse_const_int_decl() {
    let stmt = parse_stmt("const int MAX_SIZE = 100;").expect("should parse");
    match stmt.stmt {
        Statement::ConstDecl { name, ty, init } => {
            assert_eq!(name, "MAX_SIZE");
            assert_eq!(ty, Type::Int);
            assert_eq!(init.exp, Expr::Literal(Literal::Int(100)));
        }
        other => panic!("expected ConstDecl, got {:?}", other),
    }
}

/// `const float` declaration parses with the correct type and literal.
#[test]
fn test_parse_const_float_decl() {
    let stmt = parse_stmt("const float PI = 3.14159;").expect("should parse");
    match stmt.stmt {
        Statement::ConstDecl { name, ty, .. } => {
            assert_eq!(name, "PI");
            assert_eq!(ty, Type::Float);
        }
        other => panic!("expected ConstDecl, got {:?}", other),
    }
}

/// `const bool` declaration parses correctly.
#[test]
fn test_parse_const_bool_decl() {
    let stmt = parse_stmt("const bool FLAG = true;").expect("should parse");
    match stmt.stmt {
        Statement::ConstDecl { name, ty, init } => {
            assert_eq!(name, "FLAG");
            assert_eq!(ty, Type::Bool);
            assert_eq!(init.exp, Expr::Literal(Literal::Bool(true)));
        }
        other => panic!("expected ConstDecl, got {:?}", other),
    }
}

/// `const str` declaration parses correctly.
#[test]
fn test_parse_const_str_decl() {
    let stmt = parse_stmt(r#"const str GREETING = "hello";"#).expect("should parse");
    match stmt.stmt {
        Statement::ConstDecl { name, ty, init } => {
            assert_eq!(name, "GREETING");
            assert_eq!(ty, Type::Str);
            assert_eq!(init.exp, Expr::Literal(Literal::Str("hello".to_string())));
        }
        other => panic!("expected ConstDecl, got {:?}", other),
    }
}

/// A `const` declaration whose initialiser is an arithmetic expression parses.
#[test]
fn test_parse_const_expr_initialiser() {
    let stmt = parse_stmt("const int DOUBLE = 2 * 21;").expect("should parse");
    match stmt.stmt {
        Statement::ConstDecl { name, ty, .. } => {
            assert_eq!(name, "DOUBLE");
            assert_eq!(ty, Type::Int);
        }
        other => panic!("expected ConstDecl, got {:?}", other),
    }
}

/// Omitting the semicolon makes the parse fail.
#[test]
fn test_parse_const_missing_semicolon_fails() {
    assert!(
        all_consuming(statement)("const int X = 5").is_err(),
        "missing semicolon should fail to parse"
    );
}

/// Omitting the initialiser (`const int X;`) is not valid MiniC syntax.
#[test]
fn test_parse_const_missing_init_fails() {
    assert!(
        parse_stmt("const int X;").is_err(),
        "const without initialiser should fail to parse"
    );
}

/// `const` is not a valid type keyword, so `const int x` without `const`
/// keyword would be a plain `Decl`, not a `ConstDecl`.
#[test]
fn test_parse_mutable_decl_is_not_const() {
    let stmt = parse_stmt("int x = 5;").expect("should parse");
    match stmt.stmt {
        Statement::Decl { .. } => { /* correct */ }
        Statement::ConstDecl { .. } => panic!("plain Decl must not produce ConstDecl"),
        other => panic!("unexpected statement: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 1.2 Global const declarations (top-level, outside any function)
// ---------------------------------------------------------------------------

/// A top-level `const` before `main` is accepted by the program parser and
/// appears in `Program::constants`.
#[test]
fn test_parse_global_const_int() {
    let src = "const int MAX_SIZE = 100;\nvoid main() {}";
    let (_, prog) = all_consuming(program)(src).expect("should parse");
    assert_eq!(
        prog.constants.len(),
        1,
        "program should have one global constant"
    );
    match &prog.constants[0].stmt {
        Statement::ConstDecl { name, ty, .. } => {
            assert_eq!(name, "MAX_SIZE");
            // FIXED: Removed the '*' prefix operator so it compares references cleanly
            assert_eq!(ty, &Type::Int); 
        }
        other => panic!("expected ConstDecl, got {:?}", other),
    }
}

/// Multiple global consts before `main` are all collected.
#[test]
fn test_parse_multiple_global_consts() {
    let src = "const int N = 10;\nconst float PI = 3.14;\nvoid main() {}";
    let (_, prog) = all_consuming(program)(src).expect("should parse");
    assert_eq!(prog.constants.len(), 2, "expected two global constants");
}

/// A program with *no* global consts still has an empty `constants` list.
#[test]
fn test_parse_no_global_consts() {
    let src = "void main() {}";
    let (_, prog) = all_consuming(program)(src).expect("should parse");
    assert_eq!(prog.constants.len(), 0);
}

// ===========================================================================
// 2. TYPE CHECKER TESTS
// ===========================================================================
//
// Strategy: build small programs as inline strings, call `parse_and_type_check`,
// and assert on Ok / Err with message inspection.

// ---------------------------------------------------------------------------
// 2.1 Valid uses of const
// ---------------------------------------------------------------------------

/// A local `const int` declaration inside `main` is well-typed.
#[test]
fn test_typecheck_const_int_local_ok() {
    let result = parse_and_type_check(
        "void main() { const int N = 42; }",
    );
    assert!(result.is_ok(), "const int should type-check: {:?}", result);
}

/// A local `const float` inside a function is well-typed.
#[test]
fn test_typecheck_const_float_local_ok() {
    let result = parse_and_type_check(
        "void main() { const float PI = 3.14159; }",
    );
    assert!(result.is_ok(), "const float should type-check: {:?}", result);
}

/// Reading a local const inside an expression is well-typed.
#[test]
fn test_typecheck_const_used_in_expression() {
    let result = parse_and_type_check(
        "void main() { const int N = 10; int doubled = N * 2; }",
    );
    assert!(
        result.is_ok(),
        "using a const in an expression should be ok: {:?}",
        result
    );
}

/// A const can be passed as a function argument.
#[test]
fn test_typecheck_const_passed_as_argument() {
    let result = parse_and_type_check(
        r#"
        int double(int x) { return x * 2; }
        void main() { const int N = 5; int r = double(N); }
        "#,
    );
    assert!(
        result.is_ok(),
        "passing a const as argument should type-check: {:?}",
        result
    );
}

/// A global `const int` is visible (and well-typed) inside `main`.
#[test]
fn test_typecheck_global_const_visible_in_main() {
    let result = parse_and_type_check(
        "const int MAX = 100;\nvoid main() { int x = MAX; }",
    );
    assert!(
        result.is_ok(),
        "global const should be visible in main: {:?}",
        result
    );
}

/// A global const is visible in every user-defined function, not just `main`.
#[test]
fn test_typecheck_global_const_visible_in_all_functions() {
    let result = parse_and_type_check(
        r#"
        const int LIMIT = 50;
        int capped(int x) {
            if x > LIMIT { return LIMIT; }
            return x;
        }
        void main() { int r = capped(200); }
        "#,
    );
    assert!(
        result.is_ok(),
        "global const should be visible in user functions: {:?}",
        result
    );
}

/// Int/float coercion still applies to const initialisers.
#[test]
fn test_typecheck_const_float_from_int_expr_ok() {
    let result = parse_and_type_check(
        "void main() { const float half = 1 + 0.5; }",
    );
    assert!(
        result.is_ok(),
        "int+float initialiser should coerce to float: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// 2.2 Type errors involving const
// ---------------------------------------------------------------------------

/// Assigning to a local const variable is a type error.
#[test]
fn test_typecheck_const_local_assign_rejected() {
    let result = parse_and_type_check(
        "void main() { const int N = 10; N = 20; }",
    );
    assert!(
        result.is_err(),
        "assignment to a local const must be rejected"
    );
}

/// The error message for assigning to a const mentions "const" or "constant".
#[test]
fn test_typecheck_const_assign_error_message() {
    let err = parse_and_type_check("void main() { const int N = 10; N = 20; }")
        .unwrap_err();
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("const") || msg.contains("constant") || msg.contains("immutable"),
        "error message '{}' should mention const/constant/immutable",
        err.message
    );
}

/// Assigning to a global const inside `main` is a type error.
#[test]
fn test_typecheck_global_const_assign_rejected() {
    let result = parse_and_type_check(
        "const int MAX = 100;\nvoid main() { MAX = 200; }",
    );
    assert!(
        result.is_err(),
        "assignment to a global const must be rejected"
    );
}

/// Assigning to a global const inside a non-main function is also a type error.
#[test]
fn test_typecheck_global_const_assign_rejected_in_other_function() {
    let result = parse_and_type_check(
        r#"
        const int LIMIT = 10;
        void reset() { LIMIT = 0; }
        void main() {}
        "#,
    );
    assert!(
        result.is_err(),
        "assignment to global const in a non-main function must be rejected"
    );
}

/// Type mismatch in a const initialiser is a type error.
/// `const int x = true;` — Bool is not compatible with Int.
#[test]
fn test_typecheck_const_init_type_mismatch() {
    let result = parse_and_type_check(
        "void main() { const int X = true; }",
    );
    assert!(
        result.is_err(),
        "bool initialiser for const int must be a type error"
    );
}

/// `const void x = …` should be rejected (void cannot be a variable type).
#[test]
fn test_typecheck_const_void_type_rejected() {
    // The parser may reject this before the type checker, either is fine.
    let src = "void main() { const void X = 1; }";
    let result = parse_and_type_check(src);
    assert!(
        result.is_err(),
        "const void variable should be rejected"
    );
}

// ===========================================================================
// 3. INTERPRETER TESTS
// ===========================================================================
//
// Strategy: use `run(src)` (full pipeline) to check that correct programs
// execute without errors. Constants are stored as ordinary values —
// enforcement is purely static.

// ---------------------------------------------------------------------------
// 3.1 Local const at function scope
// ---------------------------------------------------------------------------

/// A local const can be declared and its value used in an expression.
#[test]
fn test_interp_const_int_local_value_accessible() {
    let src = r#"
        void main() {
            const int N = 7;
            int doubled = N * 2;
            print(doubled);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// A local const float is accessible and usable.
#[test]
fn test_interp_const_float_local() {
    let src = r#"
        void main() {
            const float PI = 3.14159;
            float area = PI * 2.0;
            print(area);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// A local const bool is accessible and can be used as an `if` condition.
#[test]
fn test_interp_const_bool_as_condition() {
    let src = r#"
        void main() {
            const bool ALWAYS = true;
            if ALWAYS { print(1); }
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// A local const inside a function can be returned.
#[test]
fn test_interp_const_return_value() {
    let src = r#"
        int answer() {
            const int LIFE = 42;
            return LIFE;
        }
        void main() {
            int r = answer();
            print(r);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 3.2 Global const declarations
// ---------------------------------------------------------------------------

/// A top-level const is accessible inside `main` at runtime.
#[test]
fn test_interp_global_const_accessible_in_main() {
    let src = r#"
        const int MAX_SIZE = 100;
        void main() {
            int x = MAX_SIZE;
            print(x);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// A top-level const is accessible inside a non-main user function.
#[test]
fn test_interp_global_const_accessible_in_user_function() {
    let src = r#"
        const float PI = 3.14159;
        float circle_area(float r) {
            return PI * r * r;
        }
        void main() {
            float a = circle_area(1.0);
            print(a);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// Multiple global consts are all accessible at runtime.
#[test]
fn test_interp_multiple_global_consts() {
    let src = r#"
        const int A = 3;
        const int B = 4;
        int hypotenuse_sq() { return A * A + B * B; }
        void main() {
            int r = hypotenuse_sq();
            print(r);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// Global consts are evaluated before any function body runs: a function that
/// immediately reads a global const gets the expected value.
#[test]
fn test_interp_global_const_evaluated_before_functions() {
    let src = r#"
        const int SEED = 42;
        int get_seed() { return SEED; }
        void main() {
            int s = get_seed();
            print(s);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 3.3 Interactions with other language features
// ---------------------------------------------------------------------------

/// A const in an outer block is visible in an inner block.
#[test]
fn test_interp_const_visible_in_nested_block() {
    let src = r#"
        void main() {
            const int OUTER = 10;
            {
                int inner = OUTER + 5;
                print(inner);
            }
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// A mutable variable can be initialised from a const, then reassigned.
#[test]
fn test_interp_mutable_var_init_from_const() {
    let src = r#"
        void main() {
            const int BASE = 1;
            int counter = BASE;
            counter = counter + 1;
            print(counter);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// A const can be used as the bound in a `while` loop condition.
#[test]
fn test_interp_const_used_as_while_bound() {
    let src = r#"
        void main() {
            const int LIMIT = 5;
            int i = 0;
            while i < LIMIT { i = i + 1; }
            print(i);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

/// A global const float is usable in arithmetic with a local variable.
#[test]
fn test_interp_global_const_in_arithmetic() {
    let src = r#"
        const float TAX_RATE = 0.2;
        float tax(float price) { return price * TAX_RATE; }
        void main() {
            float t = tax(100.0);
            print(t);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}
