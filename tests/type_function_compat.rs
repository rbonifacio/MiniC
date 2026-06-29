use mini_c::{interpreter::interpret, parser::program, semantic::type_check};

fn type_check_src(src: &str) -> Result<(), String> {
    let (_, unchecked) = program(src).map_err(|e| format!("parse error: {:?}", e))?;
    type_check(&unchecked).map(|_| ()).map_err(|e| format!("type error: {}", e.message))
}

fn run_src(src: &str) -> Result<(), String> {
    let (_, unchecked) = program(src).map_err(|e| format!("parse error: {:?}", e))?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
}

// ---------------------------------------------------------------------------
// ✅ CASOS VÁLIDOS
// ---------------------------------------------------------------------------

#[test]
fn test_valid_uninitialized_fn_decl() {
    let src = r#"
        void main() {
            fn(int) -> int f;
        }
    "#;
    assert!(type_check_src(src).is_ok(), "Expected valid uninitialized fn declaration to pass type checker");
}

#[test]
fn test_valid_fn_assign_and_call() {
    let src = r#"
        void main() {
            fn(int) -> int f;
            f = fn(int x) -> int { return x * 3; };
            int r = f(4);
            print(r);
        }
    "#;
    assert!(run_src(src).is_ok(), "Expected valid function assignment and call to succeed at runtime");
}

// ---------------------------------------------------------------------------
// ❌ CASOS INVÁLIDOS
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_uninitialized_int_decl() {
    let src = r#"
        void main() {
            int x;
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "Expected type checker to reject uninitialized int variable");
    assert!(res.unwrap_err().contains("must be initialized"));
}

#[test]
fn test_incompatible_fn_type_assignment() {
    let src = r#"
        void main() {
            fn(float) -> int f;
            f = fn(int x) -> int { return x; };
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "Expected type checker to reject incompatible function type assignment");
    assert!(res.unwrap_err().contains("assignment to f"));
}

#[test]
fn test_incompatible_fn_type_declaration() {
    let src = r#"
        void main() {
            fn(float) -> int f = fn(int x) -> int { return x; };
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "Expected type checker to reject incompatible function type initialization");
    assert!(res.unwrap_err().contains("declaration of f"));
}

#[test]
fn test_call_non_function() {
    let src = r#"
        void main() {
            int x = 42;
            x(5);
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "Expected type checker to reject calling a non-function value");
    assert!(res.unwrap_err().contains("is not a function"));
}

#[test]
fn test_call_non_function_literal() {
    let src = r#"
        void main() {
            int y = 1(5);
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "Expected type checker to reject calling a non-function literal");
    assert!(res.unwrap_err().contains("attempting to call a non-function value"));
}

#[test]
fn test_wrong_argument_count() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int { return x; };
            f(1, 2);
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "Expected type checker to reject calling function with wrong number of arguments");
    assert!(res.unwrap_err().contains("expects 1 arguments, got 2"));
}

#[test]
fn test_uninitialized_fn_call_runtime_error() {
    let src = r#"
        void main() {
            fn(int) -> int f;
            f(5);
        }
    "#;
    let res = run_src(src);
    assert!(res.is_err(), "Expected calling an uninitialized function variable to fail at runtime");
    assert!(res.unwrap_err().contains("is not a function"));
}
