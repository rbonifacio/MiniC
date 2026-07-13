use mini_c::ir::ast::{Expr, Literal, Statement};
use mini_c::parser::{expression, statement};

#[test]
fn array_literal() {
    let result = expression("[1, 2, 3]").unwrap().1;
    assert!(matches!(result.exp, Expr::ArrayLit(ref elems) if elems.len() == 3));
}

#[test]
fn empty_array() {
    let result = expression("[]").unwrap().1;
    assert!(matches!(result.exp, Expr::ArrayLit(ref elems) if elems.is_empty()));
}

#[test]
fn index_read() {
    let result = expression("arr[i]").unwrap().1;
    assert!(matches!(result.exp, Expr::Index { ref base, ref index }
        if matches!(base.exp, Expr::Ident(ref s) if s == "arr") && matches!(index.exp, Expr::Ident(ref s) if s == "i")));
}

#[test]
fn indexed_assignment() {
    let result = statement("arr[i] = 1;").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Index { .. }) && value.exp == Expr::Literal(Literal::Int(1)))
    );
}

#[test]
fn multidimensional_indexed_assignment() {
    let result = statement("arr[i][j] = x;").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Index { .. }) && matches!(value.exp, Expr::Ident(ref s) if s == "x"))
    );
}

#[test]
fn nested_index() {
    let result = expression("arr[i][j]").unwrap().1;
    assert!(matches!(result.exp, Expr::Index { ref base, ref index }
        if matches!(index.exp, Expr::Ident(ref s) if s == "j")));
}

#[test]
fn array_in_expression() {
    let result = expression("[1, 2][0]").unwrap().1;
    assert!(matches!(result.exp, Expr::Index { ref base, ref index }
        if matches!(base.exp, Expr::ArrayLit(_)) && index.exp == Expr::Literal(Literal::Int(0))));
}
