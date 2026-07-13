use mini_c::ir::ast::{Expr, Literal, Statement};
use mini_c::parser::{assignment, expression};
use nom::combinator::all_consuming;

#[test]
fn member_access_expression() {
    let result = expression("point.x").unwrap().1;
    assert!(matches!(result.exp, Expr::Member { ref base, ref member }
        if matches!(base.exp, Expr::Ident(ref s) if s == "point") && member == "x"));
}

#[test]
fn chained_member_access() {
    let result = expression("root.left.value").unwrap().1;
    assert!(matches!(result.exp, Expr::Member { ref member, .. } if member == "value"));
}

#[test]
fn member_access_with_index_expression() {
    let result = expression("items.head[0]").unwrap().1;
    assert!(matches!(result.exp, Expr::Index { ref base, .. }
        if matches!(base.exp, Expr::Member { ref member, .. } if member == "head")));
}

#[test]
fn member_assignment_target() {
    let result = assignment("p.x = 1;").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Member { ref member, .. } if member == "x") && value.exp == Expr::Literal(Literal::Int(1)))
    );
}

#[test]
fn invalid_member_access_trailing_dot() {
    assert!(all_consuming(expression)("point.").is_err());
}
