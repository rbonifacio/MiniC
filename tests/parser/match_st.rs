use mini_c::ir::ast::Statement;
use mini_c::parser::statement;

#[test]
fn match_statement() {
    let result = statement("match x { case Some: { y = Some; } case None: { z = 0; } }")
        .unwrap()
        .1;
    assert!(matches!(result.stmt, Statement::Match { .. }));
}

#[test]
fn match_single_arm() {
    let result = statement("match flag { case OK: { print(1); } }")
        .unwrap()
        .1;
    assert!(matches!(result.stmt, Statement::Match { ref arms, .. } if arms.len() == 1));
}

#[test]
fn reject_empty_arms() {
    assert!(statement("match x { }").is_err());
}

#[test]
fn reject_no_match_keyword() {
    assert!(statement("atch x { case A: { } }").is_err());
}

#[test]
fn reject_missing_case_keyword() {
    assert!(statement("match x { A: { } }").is_err());
}

#[test]
fn reject_missing_colon() {
    assert!(statement("match x { case A { } }").is_err());
}
