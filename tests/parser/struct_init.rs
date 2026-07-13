use mini_c::ir::ast::Expr;
use mini_c::parser::expression;
use nom::combinator::all_consuming;

#[test]
fn struct_init_expression() {
    let result = expression("{ .x = 1, .y = 2 }").unwrap().1;
    assert!(matches!(result.exp, Expr::Init { ref fields } if fields.len() == 2));
}

#[test]
fn struct_init_single_field() {
    let result = expression(r#"{ .name = "hello" }"#).unwrap().1;
    assert!(matches!(result.exp, Expr::Init { ref fields } if fields.len() == 1));
}

#[test]
fn struct_init_empty_rejected() {
    assert!(all_consuming(expression)("{}").is_err());
}

#[test]
fn struct_init_missing_equals() {
    assert!(all_consuming(expression)("{ .x 1 }").is_err());
}

#[test]
fn struct_init_missing_dot() {
    assert!(all_consuming(expression)("{ x = 1 }").is_err());
}
