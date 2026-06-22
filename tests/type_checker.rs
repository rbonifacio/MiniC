//! Integration tests for the MiniC type checker.

use nom::combinator::all_consuming;
use mini_c::ir::ast::{CheckedProgram, Type};
use mini_c::parser::program;
use mini_c::semantic::type_check;

fn parse_and_type_check(src: &str) -> Result<CheckedProgram, mini_c::semantic::TypeError> {
    let (_, prog) = all_consuming(program)(src).map_err(|_| {
        mini_c::semantic::TypeError {
            message: "parse failed".to_string(),
        }
    })?;
    type_check(&prog)
}

#[test]
fn test_type_check_simple_assign() {
    let result = parse_and_type_check("void main() int x = 1;");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_int_float_coercion() {
    let result = parse_and_type_check("void main() float x = 1 + 3.14;");
    assert!(result.is_ok());
    let prog = result.unwrap();
    let main_fn = prog.functions.iter().find(|f| f.name == "main").unwrap();
    if let mini_c::ir::ast::Statement::Decl { ref init, .. } = main_fn.body.stmt {
        assert_eq!(init.ty, Type::Float);
    } else {
        panic!("expected Decl");
    }
}

#[test]
fn test_type_check_undeclared_var() {
    let result = parse_and_type_check("void main() x = y;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("undeclared"));
}

#[test]
fn test_type_check_bool_condition() {
    let result = parse_and_type_check("void main() if true { int x = 1; }");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_array_literal() {
    let result = parse_and_type_check("void main() int[] x = [1, 2, 3];");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_array_index() {
    let result = parse_and_type_check("void main() { int[] arr = [1, 2]; int x = arr[0]; }");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_call_arg_type_mismatch() {
    let result = parse_and_type_check("void foo(int x) x = x;\nvoid main() foo(true);");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("argument"));
}

#[test]
fn test_type_check_missing_main() {
    let result = parse_and_type_check("void foo() int x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("main"));
}

#[test]
fn test_type_check_decl_then_assign() {
    let result = parse_and_type_check("void main() { int x = 1; x = 2; }");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_assign_undeclared() {
    let result = parse_and_type_check("void main() x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("undeclared"));
}

#[test]
fn test_type_check_redeclaration() {
    let result = parse_and_type_check("void main() { int x = 1; int x = 2; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("redeclaration"));
}

#[test]
fn test_type_check_relational_type_mismatch() {
    let result = parse_and_type_check("void main() bool r = \"hello\" == 42;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("compatible"));
}

#[test]
fn test_type_check_ordering_requires_numeric() {
    let result = parse_and_type_check("void main() bool r = true < false;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("numeric"));
}

#[test]
fn test_type_check_equality_same_type_ok() {
    let result = parse_and_type_check("void main() bool r = 1 == 2;");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_main_non_void_return() {
    let result = parse_and_type_check("int main() int x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("void"));
}

#[test]
fn test_type_check_main_with_params() {
    let result = parse_and_type_check("void main(int x) int y = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("parameters"));
}

#[test]
fn test_type_check_return_void_ok() {
    let result = parse_and_type_check("void main() { int x = 1; return; }");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_return_value_in_void_fn() {
    let result = parse_and_type_check("void main() return 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("void function must not return a value"));
}

#[test]
fn test_type_check_return_correct_type() {
    let result = parse_and_type_check("int foo() return 42;\nvoid main() int x = 1;");
    assert!(result.is_ok());
}

#[test]
fn test_type_check_return_wrong_type() {
    let result = parse_and_type_check("int foo() return true;\nvoid main() int x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("return type mismatch"));
}

#[test]
fn test_type_check_block_scoping() {
    // variable declared inside a block should not be visible after the block
    let result = parse_and_type_check("void main() { { int x = 1; } x = 2; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("undeclared"));
}

// ---------------------------------------------------------------------------
// 7.2 print accepts any type via Type::Any
// ---------------------------------------------------------------------------
#[test]
fn test_type_check_print_accepts_int() {
    assert!(parse_and_type_check("void main() { print(42); }").is_ok());
}

#[test]
fn test_type_check_print_accepts_bool() {
    assert!(parse_and_type_check("void main() { print(true); }").is_ok());
}

#[test]
fn test_type_check_print_accepts_str() {
    assert!(parse_and_type_check("void main() { print(\"hello\"); }").is_ok());
}

#[test]
fn test_type_check_print_accepts_float() {
    assert!(parse_and_type_check("void main() { print(3.14); }").is_ok());
}

#[test]
fn test_type_check_print_accepts_array() {
    assert!(parse_and_type_check("void main() { print([1, 2, 3]); }").is_ok());
}

// ---------------------------------------------------------------------------
// 7.3 Wrong arity on stdlib function reports type error
// ---------------------------------------------------------------------------
#[test]
fn test_type_check_sqrt_wrong_arity() {
    let result = parse_and_type_check("void main() { float r = sqrt(); }");
    assert!(result.is_err(), "expected arity error for sqrt()");
}

#[test]
fn test_type_check_print_wrong_arity() {
    let result = parse_and_type_check("void main() { print(1, 2); }");
    assert!(result.is_err(), "expected arity error for print(1, 2)");
}
// =============================================================================
// PATCH 2 — tests/type_checker.rs
// Milestone 2: Type Checker
// ---------------------------------------------------------------------------
// Const declarations — valid programs
// ---------------------------------------------------------------------------

/// A local `const int` inside `main` is well-typed.
#[test]
fn test_const_local_int_ok() {
    assert!(parse_and_type_check("void main() { const int N = 42; }").is_ok());
}

/// A local `const float` inside a function is well-typed.
#[test]
fn test_const_local_float_ok() {
    assert!(parse_and_type_check("void main() { const float PI = 3.14159; }").is_ok());
}

/// A const can be read in an expression inside the same scope.
#[test]
fn test_const_used_in_expression() {
    assert!(parse_and_type_check(
        "void main() { const int N = 10; int doubled = N * 2; }"
    ).is_ok());
}

/// A const can be passed as a function argument.
#[test]
fn test_const_passed_as_argument() {
    assert!(parse_and_type_check(r#"
        int double(int x) { return x * 2; }
        void main() { const int N = 5; int r = double(N); }
    "#).is_ok());
}

/// Int/float coercion applies to const initialisers.
#[test]
fn test_const_float_from_int_expr_coercion() {
    assert!(parse_and_type_check(
        "void main() { const float half = 1 + 0.5; }"
    ).is_ok());
}

/// A global `const int` is visible inside `main`.
#[test]
fn test_const_global_visible_in_main() {
    assert!(parse_and_type_check(
        "const int MAX = 100;\nvoid main() { int x = MAX; }"
    ).is_ok());
}

/// A global const is visible in every user-defined function, not just `main`.
#[test]
fn test_const_global_visible_in_all_functions() {
    assert!(parse_and_type_check(r#"
        const int LIMIT = 50;
        int capped(int x) {
            if x > LIMIT { return LIMIT; }
            return x;
        }
        void main() { int r = capped(200); }
    "#).is_ok());
}

// ---------------------------------------------------------------------------
// Const declarations — type errors
// ---------------------------------------------------------------------------

/// Assigning to a local const is a type error.
#[test]
fn test_const_local_assign_rejected() {
    assert!(
        parse_and_type_check("void main() { const int N = 10; N = 20; }").is_err(),
        "assignment to a local const must be rejected"
    );
}

/// The error message for assigning to a const mentions immutability.
#[test]
fn test_const_assign_error_message_mentions_const() {
    let err = parse_and_type_check("void main() { const int N = 10; N = 20; }")
        .unwrap_err();
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("const") || msg.contains("constant") || msg.contains("immutable"),
        "error '{}' should mention const/constant/immutable",
        err.message
    );
}

/// Assigning to a global const inside `main` is a type error.
#[test]
fn test_const_global_assign_rejected_in_main() {
    assert!(
        parse_and_type_check("const int MAX = 100;\nvoid main() { MAX = 200; }").is_err(),
        "assignment to global const must be rejected"
    );
}

/// Assigning to a global const inside a non-main function is also a type error.
#[test]
fn test_const_global_assign_rejected_in_other_function() {
    assert!(
        parse_and_type_check(r#"
            const int LIMIT = 10;
            void reset() { LIMIT = 0; }
            void main() {}
        "#).is_err(),
        "assignment to global const in any function must be rejected"
    );
}

/// A type mismatch in a const initialiser is a type error (`const int x = true`).
#[test]
fn test_const_init_type_mismatch() {
    assert!(
        parse_and_type_check("void main() { const int X = true; }").is_err(),
        "bool initialiser for const int must be a type error"
    );
}

/// `const void x = …` is rejected (void cannot be a variable type).
#[test]
fn test_const_void_type_rejected() {
    assert!(
        parse_and_type_check("void main() { const void X = 1; }").is_err(),
        "const void must be rejected"
    );
}