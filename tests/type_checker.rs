//! Integration tests for the MiniC type checker.

use std::path::Path;

use nom::combinator::all_consuming;
use mini_c::ir::ast::{CheckedProgram, Expr, Statement, Type};
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

fn parse_and_type_check_fixture(name: &str) -> Result<CheckedProgram, mini_c::semantic::TypeError> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let src = std::fs::read_to_string(&path).expect("fixture file should exist");
    parse_and_type_check(src.trim())
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
// Pointers
// ---------------------------------------------------------------------------
#[test]
fn test_type_check_pointer_init_fixture() {
    assert!(
        parse_and_type_check_fixture("pointer_init.minic").is_ok(),
        "{}",
        parse_and_type_check_fixture("pointer_init.minic")
            .unwrap_err()
            .message
    );
}

#[test]
fn test_type_check_pointer_feature_fixture() {
    assert!(parse_and_type_check_fixture("pointer_feature.minic").is_ok());
}

#[test]
fn test_type_check_pointer_function_fixture() {
    assert!(parse_and_type_check_fixture("pointer_function.minic").is_ok());
}

#[test]
fn test_type_check_addr_of_and_deref_types() {
    let prog = parse_and_type_check(
        "void main() { int x = 1; int* p = &x; int y = *p; }",
    )
    .expect("pointer decl and deref should type-check");

    let main_fn = prog.functions.iter().find(|f| f.name == "main").unwrap();
    let Statement::Block { seq } = &main_fn.body.stmt else {
        panic!("expected block body");
    };

    let p_init = match &seq[1].stmt {
        Statement::Decl { init, .. } => init,
        _ => panic!("expected int* p = &x decl"),
    };
    assert_eq!(p_init.ty, Type::Pointer(Box::new(Type::Int)));
    assert!(matches!(p_init.exp, Expr::AddrOf(_)));

    let y_init = match &seq[2].stmt {
        Statement::Decl { init, .. } => init,
        _ => panic!("expected int y = *p decl"),
    };
    assert_eq!(y_init.ty, Type::Int);
    assert!(matches!(y_init.exp, Expr::Deref(_)));
}

#[test]
fn test_type_check_pointer_param_and_deref_assign() {
    let prog = parse_and_type_check(
        "void bump(int* p) { *p = *p + 1; }\nvoid main() { int x = 0; bump(&x); }",
    )
    .expect("pointer param and assignment through deref");

    let bump = prog.functions.iter().find(|f| f.name == "bump").unwrap();
    assert_eq!(
        bump.params,
        vec![("p".to_string(), Type::Pointer(Box::new(Type::Int)))]
    );
}

#[test]
fn test_type_check_pointer_return_type() {
    let prog = parse_and_type_check(
        "int* id(int* p) { return p; }\nvoid main() { int x = 1; int* q = id(&x); }",
    )
    .expect("pointer return");

    let id_fn = prog.functions.iter().find(|f| f.name == "id").unwrap();
    assert_eq!(id_fn.return_type, Type::Pointer(Box::new(Type::Int)));
}

#[test]
fn test_type_check_pointer_rebind() {
    assert!(parse_and_type_check(
        "void main() { int x = 1; int y = 2; int* a = &x; int* b = &y; a = b; }",
    )
    .is_ok());
}

#[test]
fn test_type_check_deref_non_pointer() {
    let result = parse_and_type_check("void main() { int x = 1; int y = *x; }");
    assert!(result.is_err());
    assert!(
        result.unwrap_err().message.contains("cannot dereference non-pointer")
    );
}

#[test]
fn test_type_check_addr_of_non_variable() {
    let result = parse_and_type_check("void main() { int x = 1; int* p = &(x + 1); }");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .message
            .contains("can only take address of variables")
    );
}

#[test]
fn test_type_check_deref_assign_type_mismatch() {
    let result = parse_and_type_check(
        "void main() { int x = 1; int* p = &x; *p = true; }",
    );
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .message
            .contains("assignment through pointer")
    );
}

#[test]
fn test_type_check_call_expects_pointer_arg() {
    let result = parse_and_type_check(
        "void take(int* p) { *p = 1; }\nvoid main() { int x = 0; take(x); }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("argument"));
}

#[test]
fn test_type_check_assign_int_from_pointer() {
    let result = parse_and_type_check(
        "void main() { int x = 1; int* p = &x; int y = p; }",
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.message.contains("declaration"),
        "unexpected error: {}",
        err.message
    );
}
