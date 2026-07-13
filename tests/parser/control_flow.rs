use mini_c::ir::ast::Statement;
use mini_c::parser::statement;

// --- If/Else ---

#[test]
fn if_without_else() {
    let result = statement("if x { y = 1; }").unwrap().1;
    assert!(matches!(
        result.stmt,
        Statement::If {
            else_branch: None,
            ..
        }
    ));
}

#[test]
fn if_with_else() {
    let result = statement("if x { y = 1; } else { y = 0; }").unwrap().1;
    assert!(matches!(
        result.stmt,
        Statement::If {
            else_branch: Some(_),
            ..
        }
    ));
}

#[test]
fn nested_if() {
    let result = statement("if a { if b { x = 1; } else { x = 2; } }")
        .unwrap()
        .1;
    assert!(matches!(result.stmt, Statement::If { .. }));
}

#[test]
fn if_whitespace() {
    assert!(statement("if  x  { y  =  1; }").is_ok());
}

#[test]
fn invalid_if() {
    assert!(statement("if x").is_err());
    assert!(statement("if x y = 1;").is_err());
}

// --- While ---

#[test]
fn simple_while() {
    let result = statement("while x { y = 1; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
}

#[test]
fn while_with_expression() {
    let result = statement("while i < 10 { i = i + 1; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
}

#[test]
fn nested_while() {
    let result = statement("while a { while b { x = 1; } }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
}

#[test]
fn while_whitespace() {
    assert!(statement("while  x  { y  =  1; }").is_ok());
}

#[test]
fn invalid_while() {
    assert!(statement("while x").is_err());
    assert!(statement("while x y = 1;").is_err());
}
