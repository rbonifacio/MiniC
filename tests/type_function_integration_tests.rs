use mini_c::parser::functions::{fun_decl, type_name};
use mini_c::parser::statements::statement;

// =========================
// ✅ DECLARAÇÕES COM TYPE::FUN
// =========================

#[test]
fn test_statement_decl_with_function_type_no_init() {
    let input = "fn(int) -> int f;";

    let result = statement(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

#[test]
fn test_statement_decl_with_nested_function_type_no_init() {
    let input = "fn(fn(int) -> int) -> int g;";

    let result = statement(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

#[test]
fn test_statement_decl_with_function_type_and_init() {
    let input = "fn(int) -> int f = foo;";

    let result = statement(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

// =========================
// ✅ FUNÇÕES COM PARÂMETROS TYPE::FUN
// =========================

#[test]
fn test_fun_decl_param_with_function_type() {
    let input = "int apply(fn(int) -> int f, int x) { return x; }";

    let result = fun_decl(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

#[test]
fn test_fun_decl_nested_function_type_in_param() {
    let input = "int h(fn(fn(int) -> int) -> int f) { return 0; }";

    let result = fun_decl(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

// =========================
// ✅ FUNÇÕES COM RETORNO TYPE::FUN
// =========================

#[test]
fn test_fun_decl_returning_function_type() {
    let input = "fn(int) -> int make() { return foo; }";

    let result = fun_decl(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

#[test]
fn test_fun_decl_returning_nested_function_type() {
    let input = "fn(int) -> fn(int) -> int make() { return foo; }";

    let result = fun_decl(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

// =========================
// ✅ ESPAÇOS / ROBUSTEZ
// =========================

#[test]
fn test_fun_decl_with_many_spaces() {
    let input = "
        int   apply(
            fn(int) -> int   f,
            int   x
        )
        {
            return x;
        }
    ";

    let result = fun_decl(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

#[test]
fn test_statement_decl_with_many_spaces() {
    let input = "
        fn( int , float ) -> bool    g   ;
    ";

    let result = statement(input);
    assert!(result.is_ok());

    let (rest, _) = result.unwrap();
    assert_eq!(rest.trim(), "");
}

// =========================
// ❌ NÃO CONFUNDIR COM LAMBDA
// =========================

#[test]
fn test_type_name_still_rejects_lambda() {
    let input = "fn(int x) -> int { return x; }";

    let result = type_name(input);
    assert!(result.is_err());
}