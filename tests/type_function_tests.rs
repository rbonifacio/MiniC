use mini_c::parser::functions::type_name;
use mini_c::ir::ast::Type;

// =========================
// ✅ CASOS VÁLIDOS
// =========================

#[test]
fn test_fun_type_simple() {
    let input = "fn(int) -> int";

    let result = type_name(input);
    assert!(result.is_ok());

    let (rest, ty): (&str, Type) = result.unwrap();
    assert_eq!(rest.trim(), "");

    match ty {
        Type::Fun(params, ret) => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], Type::Int);
            assert_eq!(*ret, Type::Int);
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_fun_type_multiple_params() {
    let input = "fn(int, float) -> bool";

    let (_, ty) = type_name(input).unwrap();

    match ty {
        Type::Fun(params, ret) => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], Type::Int);
            assert_eq!(params[1], Type::Float);
            assert_eq!(*ret, Type::Bool);
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_fun_type_no_params() {
    let input = "fn() -> int";

    let (_, ty) = type_name(input).unwrap();

    match ty {
        Type::Fun(params, ret) => {
            assert_eq!(params.len(), 0);
            assert_eq!(*ret, Type::Int);
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_fun_type_nested() {
    let input = "fn(fn(int) -> int) -> int";

    let (_, ty) = type_name(input).unwrap();

    match ty {
        Type::Fun(params, ret) => {
            assert_eq!(params.len(), 1);

            match &params[0] {
                Type::Fun(inner_params, inner_ret) => {
                    assert_eq!(inner_params.len(), 1);
                    assert_eq!(inner_params[0], Type::Int);
                    assert_eq!(**inner_ret, Type::Int);
                }
                _ => panic!("Expected nested function type"),
            }

            assert_eq!(*ret, Type::Int);
        }
        _ => panic!("Expected function type"),
    }
}

#[test]
fn test_fun_type_spaces_everywhere() {
    let input = "   fn(  int ,   float )   ->   bool   ";

    let (_, ty) = type_name(input).unwrap();

    match ty {
        Type::Fun(params, ret) => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], Type::Int);
            assert_eq!(params[1], Type::Float);
            assert_eq!(*ret, Type::Bool);
        }
        _ => panic!("Expected function type"),
    }
}

// =========================
// ❌ CASOS INVÁLIDOS
// =========================

#[test]
fn test_lambda_not_type() {
    let input = "fn(int x) -> int { return x; }";

    let result = type_name(input);

    assert!(result.is_err());
}

#[test]
fn test_invalid_comma() {
    let input = "fn(,) -> int";

    let result = type_name(input);

    assert!(result.is_err());
}

#[test]
fn test_missing_arrow() {
    let input = "fn(int) int";

    let result = type_name(input);

    assert!(result.is_err());
}

#[test]
fn test_unclosed_paren() {
    let input = "fn(int -> int";

    let result = type_name(input);

    assert!(result.is_err());
}

#[test]
fn test_missing_return_type() {
    let input = "fn(int) ->";

    let result = type_name(input);

    assert!(result.is_err());
}

#[test]
fn test_random_garbage() {
    let input = "fn(abc) => ???";

    let result = type_name(input);

    assert!(result.is_err());
}

// =========================
// 🔥 EDGE CASES
// =========================

#[test]
fn test_deeply_nested_functions() {
    let input = "fn(fn(fn(int) -> int) -> int) -> int";

    let result = type_name(input);

    assert!(result.is_ok());
}

#[test]
fn test_trailing_input() {
    let input = "fn(int) -> int extra";

    let (rest, _): (&str, Type) = type_name(input).unwrap();

    assert!(rest.trim().starts_with("extra"));
}

// =========================
// 💡 ROBUSTEZ
// =========================

#[test]
fn test_many_spaces_and_newlines() {
    let input = "
        fn(
            int,
            float
        )
        ->
        bool
    ";

    let result = type_name(input);

    assert!(result.is_ok());
}