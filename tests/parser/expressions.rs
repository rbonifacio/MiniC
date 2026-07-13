use mini_c::ir::ast::{Expr, ExprD, Literal};
use mini_c::parser::expression;
use nom::combinator::all_consuming;

#[test]
fn primary_literal() {
    assert_eq!(
        expression("42").map(|(r, e)| (r, e.exp)),
        Ok(("", Expr::Literal(Literal::Int(42))))
    );
    assert_eq!(
        expression("true").map(|(r, e)| (r, e.exp)),
        Ok(("", Expr::Literal(Literal::Bool(true))))
    );
    assert_eq!(
        expression("x").map(|(r, e)| (r, e.exp)),
        Ok(("", Expr::Ident("x".to_string())))
    );
}

#[test]
fn arithmetic() {
    assert_eq!(
        expression("1 + 2").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Add(
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(1)),
                    ty: ()
                }),
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(2)),
                    ty: ()
                })
            )
        ))
    );
    assert_eq!(
        expression("10 - 3").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Sub(
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(10)),
                    ty: ()
                }),
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(3)),
                    ty: ()
                })
            )
        ))
    );
    assert_eq!(
        expression("4 * 5").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Mul(
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(4)),
                    ty: ()
                }),
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(5)),
                    ty: ()
                })
            )
        ))
    );
    assert_eq!(
        expression("-x").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Neg(Box::new(ExprD {
                exp: Expr::Ident("x".to_string()),
                ty: ()
            }))
        ))
    );
}

#[test]
fn precedence_arithmetic() {
    let result = expression("1 + 2 * 3").unwrap().1.exp;
    match &result {
        Expr::Add(l, r) => {
            assert_eq!(l.exp, Expr::Literal(Literal::Int(1)));
            match &r.exp {
                Expr::Mul(m, n) => {
                    assert_eq!(m.exp, Expr::Literal(Literal::Int(2)));
                    assert_eq!(n.exp, Expr::Literal(Literal::Int(3)));
                }
                _ => panic!("expected Mul"),
            }
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn parentheses() {
    let result = expression("(1 + 2) * 3").unwrap().1.exp;
    match &result {
        Expr::Mul(l, r) => {
            match &l.exp {
                Expr::Add(a, b) => {
                    assert_eq!(a.exp, Expr::Literal(Literal::Int(1)));
                    assert_eq!(b.exp, Expr::Literal(Literal::Int(2)));
                }
                _ => panic!("expected Add"),
            }
            assert_eq!(r.exp, Expr::Literal(Literal::Int(3)));
        }
        _ => panic!("expected Mul"),
    }
}

#[test]
fn relational() {
    assert!(matches!(
        expression("a == b").unwrap().1.exp,
        Expr::Eq(_, _)
    ));
    assert!(matches!(expression("x < 5").unwrap().1.exp, Expr::Lt(_, _)));
    assert!(matches!(
        expression("1 + 2 < 5").unwrap().1.exp,
        Expr::Lt(_, _)
    ));
}

#[test]
fn complex_expression() {
    let result = expression("a >= (pi * r * r) + epsilon").unwrap().1.exp;
    assert!(matches!(result, Expr::Ge(_, _)));
}

#[test]
fn boolean_expr() {
    assert!(matches!(
        expression("true and false").unwrap().1.exp,
        Expr::And(_, _)
    ));
    assert!(matches!(expression("!x").unwrap().1.exp, Expr::Not(_)));
    assert!(matches!(
        expression("x < 5 and y > 0").unwrap().1.exp,
        Expr::And(_, _)
    ));
}

#[test]
fn invalid_trailing_op() {
    assert!(all_consuming(expression)("1 +").is_err());
}

#[test]
fn invalid_unbalanced_paren() {
    assert!(expression("(1 + 2").is_err());
    assert!(all_consuming(expression)("1 + 2)").is_err());
}
