use mini_c::ir::ast::Statement;
use mini_c::parser::{fun_decl, statement};

#[test]
fn empty_block() {
    let result = statement("{}").unwrap().1;
    assert!(matches!(result.stmt, Statement::Block { ref seq } if seq.is_empty()));
}

#[test]
fn block_single_statement() {
    let result = statement("{ x = 1; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::Block { ref seq } if seq.len() == 1));
}

#[test]
fn block_multiple_statements() {
    let result = statement("{ x = 1; y = 2; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::Block { ref seq } if seq.len() == 2));
}

#[test]
fn block_in_function_body() {
    let result = fun_decl("void foo(int x, int y) { x = x + 1; y = y + 1; }")
        .unwrap()
        .1;
    assert!(matches!(result.body.stmt, Statement::Block { ref seq } if seq.len() == 2));
}

#[test]
fn block_in_if_body() {
    let result = statement("if x { a = 1; b = 2; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::If { .. }));
}

#[test]
fn block_in_while_body() {
    let result = statement("while x { a = 1; b = 2; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
}
