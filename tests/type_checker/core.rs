use super::helpers::parse_and_type_check;
use mini_c::ir::ast::Type;

#[test]
fn simple_assign() {
    assert!(parse_and_type_check("void main() int x = 1;").is_ok());
}

#[test]
fn int_float_coercion() {
    let result = parse_and_type_check("void main() float x = 1 + 3.14;");
    assert!(result.is_ok());
    let prog = result.unwrap();
    let main_fn = prog.functions.iter().find(|f| f.name == "main").unwrap();
    if let mini_c::ir::ast::Statement::Decl { ref init, .. } = main_fn.body.stmt {
        assert_eq!(init.ty, Type::Float);
    }
}

#[test]
fn undeclared_var() {
    let result = parse_and_type_check("void main() x = y;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("undeclared"));
}

#[test]
fn bool_condition() {
    assert!(parse_and_type_check("void main() if true { int x = 1; }").is_ok());
}

#[test]
fn while_condition_must_be_bool() {
    let result = parse_and_type_check("void main() while 3 { }");
    assert!(result.is_err(), "expected error on int while condition");
}

#[test]
fn array_literal() {
    assert!(parse_and_type_check("void main() int[] x = [1, 2, 3];").is_ok());
}

#[test]
fn array_index() {
    assert!(parse_and_type_check("void main() { int[] arr = [1, 2]; int x = arr[0]; }").is_ok());
}

#[test]
fn call_arg_type_mismatch() {
    let result = parse_and_type_check("void foo(int x) x = x;\nvoid main() foo(true);");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("argument"));
}

#[test]
fn missing_main() {
    let result = parse_and_type_check("void foo() int x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("main"));
}

#[test]
fn decl_then_assign() {
    assert!(parse_and_type_check("void main() { int x = 1; x = 2; }").is_ok());
}

#[test]
fn assign_undeclared() {
    let result = parse_and_type_check("void main() x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("undeclared"));
}

#[test]
fn redeclaration() {
    let result = parse_and_type_check("void main() { int x = 1; int x = 2; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("redeclaration"));
}

#[test]
fn variable_shadowing_rejected() {
    let result = parse_and_type_check("void main() { int x = 1; { int x = 2; } }");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("redeclaration"));
}

#[test]
fn relational_type_mismatch() {
    let result = parse_and_type_check(r#"void main() bool r = "hello" == 42;"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("compatible"));
}

#[test]
fn ordering_requires_numeric() {
    let result = parse_and_type_check("void main() bool r = true < false;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("numeric"));
}

#[test]
fn equality_same_type_ok() {
    assert!(parse_and_type_check("void main() bool r = 1 == 2;").is_ok());
}

#[test]
fn main_non_void_return() {
    let result = parse_and_type_check("int main() int x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("void"));
}

#[test]
fn main_with_params() {
    let result = parse_and_type_check("void main(int x) int y = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("parameters"));
}

#[test]
fn return_void_ok() {
    assert!(parse_and_type_check("void main() { int x = 1; return; }").is_ok());
}

#[test]
fn return_value_in_void_fn() {
    let result = parse_and_type_check("void main() return 1;");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("void function must not return a value"));
}

#[test]
fn return_correct_type() {
    assert!(parse_and_type_check("int foo() return 42;\nvoid main() int x = 1;").is_ok());
}

#[test]
fn return_wrong_type() {
    let result = parse_and_type_check("int foo() return true;\nvoid main() int x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("return type mismatch"));
}

#[test]
fn block_scoping() {
    let result = parse_and_type_check("void main() { { int x = 1; } x = 2; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("undeclared"));
}

#[test]
fn array_element_assign_type_mismatch() {
    let result = parse_and_type_check("void main() { int[] arr = [1, 2]; arr[0] = true; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("type"));
}

#[test]
fn struct_field_assign_type_mismatch() {
    let result = parse_and_type_check(
        "struct Point { int x; }\nvoid main() { struct Point p = { .x = 0 }; p.x = true; }",
    );
    let err = result.unwrap_err();
    assert!(err.message.contains("expected") || err.message.contains("type"));
}
