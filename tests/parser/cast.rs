use mini_c::ir::ast::{Expr, Type};
use mini_c::parser::expression;
use nom::combinator::all_consuming;

#[test]
fn cast_expression() {
    let result = expression("(int)1.5").unwrap().1;
    assert!(matches!(result.exp, Expr::Cast { ref ty, .. } if *ty == Type::Int));
}

#[test]
fn cast_with_struct_type() {
    let result = all_consuming(expression)("(struct Point){ .x = 1 }")
        .unwrap()
        .1;
    assert!(matches!(result.exp, Expr::Cast { ref ty, .. } if matches!(ty, Type::Struct(_))));
}

#[test]
fn cast_precedence() {
    let result = expression("(int)1.5 + 2").unwrap().1;
    assert!(matches!(result.exp, Expr::Add(_, _)));
}

#[test]
fn cast_reject_non_type() {
    assert!(all_consuming(expression)("(x)1").is_err());
    assert!(all_consuming(expression)("(not_a_type)1").is_err());
}

#[test]
fn cast_reject_incomplete() {
    assert!(all_consuming(expression)("(int)").is_err());
}

#[test]
fn cast_with_enum_type() {
    let result = expression("(enum Color)Red").unwrap().1;
    assert!(matches!(result.exp, Expr::Cast { ref ty, .. } if matches!(ty, Type::Enum(_))));
}

#[test]
fn precedence_cast_vs_mul() {
    let result = expression("(int)2 * 3").unwrap().1;
    assert!(matches!(result.exp, Expr::Mul(_, _)));
}
