use mini_c::ir::ast::{Expr, Literal, Statement, Type};
use mini_c::parser::{assignment, statement};

#[test]
fn simple_assignment() {
    let result = assignment("x = 42;").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Ident(ref s) if s == "x") && value.exp == Expr::Literal(Literal::Int(42)))
    );
}

#[test]
fn assignment_with_expression() {
    let result = assignment("sum = a + b;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, .. }
        if matches!(target.exp, Expr::Ident(ref s) if s == "sum")));
}

#[test]
fn assignment_whitespace() {
    assert!(assignment("x=1;").is_ok());
    assert!(assignment("x = 1;").is_ok());
    assert!(assignment("x  =  1;").is_ok());
}

#[test]
fn invalid_assignment() {
    assert!(assignment("= 1").is_err());
    assert!(assignment("x").is_err());
    assert!(assignment("1 = x").is_err());
}

#[test]
fn decl_statement() {
    let result = statement("int x = 42;").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Decl { ref name, ref ty, .. } if name == "x" && ty == &Type::Int)
    );
    let result = statement("float y = 3.14;").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Decl { ref name, ref ty, .. } if name == "y" && ty == &Type::Float)
    );
    let result = statement("int[] arr = [1, 2, 3];").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Decl { ref name, ref ty, .. } if name == "arr" && matches!(ty, Type::Array(_)))
    );
}
