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

// ---------------------------------------------------------------------------
// For statement (Milestone 2): the type checker scopes the `init` variable to
// the loop, requires `cond` to be Bool, and validates the `update` assignment.
// ---------------------------------------------------------------------------

// A canonical, fully-formed `for` should type-check.
#[test]
fn test_type_check_for_well_typed() {
    let result = parse_and_type_check(
        "void main() { int sum = 0; for (int i = 0; i < 10; i = i + 1) { sum = sum + i; } }",
    );
    assert!(result.is_ok(), "{:?}", result.err());
}

// Every header clause may be omitted (`for (;;)`); an absent condition is
// treated as "always true" and is therefore well-typed.
#[test]
fn test_type_check_for_all_clauses_omitted() {
    let result = parse_and_type_check("void main() { for (;;) { return; } }");
    assert!(result.is_ok(), "{:?}", result.err());
}

// `init` may be a bare assignment to an already-declared variable.
#[test]
fn test_type_check_for_assign_init() {
    let result = parse_and_type_check(
        "void main() { int i = 0; for (i = 0; i < 3; i = i + 1) { i = i; } }",
    );
    assert!(result.is_ok(), "{:?}", result.err());
}

// A non-boolean condition must be rejected.
#[test]
fn test_type_check_for_non_bool_condition() {
    let result = parse_and_type_check(
        "void main() { for (int i = 0; i + 1; i = i + 1) { i = i; } }",
    );
    assert!(result.is_err(), "expected non-bool condition to be rejected");
    assert!(result.unwrap_err().message.contains("for condition must be Bool"));
}

// The variable introduced by `init` must not leak past the loop.
#[test]
fn test_type_check_for_init_var_scoped_to_loop() {
    let result = parse_and_type_check(
        "void main() { for (int i = 0; i < 3; i = i + 1) { i = i; } i = 5; }",
    );
    assert!(result.is_err(), "expected loop variable to be out of scope after the loop");
    assert!(result.unwrap_err().message.contains("undeclared"));
}

// A type error inside the update assignment must be reported.
#[test]
fn test_type_check_for_bad_update() {
    let result = parse_and_type_check(
        "void main() { for (int i = 0; i < 3; i = true) { i = i; } }",
    );
    assert!(result.is_err(), "expected ill-typed update to be rejected");
}

// --- Optional header clauses (each omitted individually) ---

#[test]
fn test_type_check_for_omitted_init() {
    let result = parse_and_type_check(
        "void main() { int i = 0; for (; i < 3; i = i + 1) { i = i; } }",
    );
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_type_check_for_omitted_cond() {
    let result = parse_and_type_check(
        "void main() { for (int i = 0; ; i = i + 1) { if i >= 3 { return; } } }",
    );
    assert!(result.is_ok(), "{:?}", result.err());
}

#[test]
fn test_type_check_for_omitted_update() {
    let result = parse_and_type_check(
        "void main() { for (int i = 0; i < 3; ) { i = i + 1; } }",
    );
    assert!(result.is_ok(), "{:?}", result.err());
}

// Assignment init requires the target to already exist in the enclosing scope.
#[test]
fn test_type_check_for_assign_init_undeclared() {
    let result = parse_and_type_check(
        "void main() { for (i = 0; i < 3; i = i + 1) { i = i; } }",
    );
    assert!(result.is_err(), "expected undeclared init assignment target");
    assert!(result.unwrap_err().message.contains("undeclared"));
}

// The loop variable from a declaration init is visible in cond, body, and update.
#[test]
fn test_type_check_for_init_var_visible_in_header() {
    let result = parse_and_type_check(
        "void main() { for (int i = 0; i < i + 10; i = i + 1) { i = i; } }",
    );
    assert!(result.is_ok(), "{:?}", result.err());
}
