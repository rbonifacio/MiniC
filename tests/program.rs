//! Integration tests for parsing MiniC programs from files.

use MiniC::ir::ast::{Program, Stmt};
use MiniC::parser::program;
use nom::combinator::all_consuming;
use std::path::Path;

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn parse_program_file(name: &str) -> Result<Program, nom::Err<nom::error::Error<String>>> {
    let path = fixtures_dir().join(name);
    let src = std::fs::read_to_string(&path).expect("fixture file should exist");
    let src = src.trim();
    let parse_result = all_consuming(program)(src);
    match parse_result {
        Ok((_, prog)) => Ok(prog),
        Err(e) => Err(e.map_input(String::from)),
    }
}

#[test]
fn test_parse_empty_program() {
    let prog = parse_program_file("empty.minic").expect("empty program should parse");
    assert!(prog.functions.is_empty());
    assert!(prog.body.is_empty());
}

#[test]
fn test_parse_statements_only() {
    let prog = parse_program_file("statements_only.minic").expect("statements-only program should parse");
    assert!(prog.functions.is_empty());
    assert_eq!(prog.body.len(), 2);
    assert!(matches!(prog.body[0], Stmt::Assign { ref target, .. } if target == "x"));
    assert!(matches!(prog.body[1], Stmt::Assign { ref target, .. } if target == "y"));
}

#[test]
fn test_parse_function_single() {
    let prog = parse_program_file("function_single.minic").expect("single-function program should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "foo");
    assert!(prog.functions[0].params.is_empty());
    assert!(matches!(prog.functions[0].body.as_ref(), Stmt::Assign { ref target, .. } if target == "x"));
    assert!(prog.body.is_empty());
}

#[test]
fn test_parse_function_with_block() {
    let prog = parse_program_file("function_with_block.minic").expect("function with block should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "add");
    assert_eq!(prog.functions[0].params, vec!["x", "y"]);
    assert!(matches!(prog.functions[0].body.as_ref(), Stmt::Block { ref seq } if seq.len() == 2));
}

#[test]
fn test_parse_full_program() {
    let prog = parse_program_file("full_program.minic").expect("full program should parse");
    assert_eq!(prog.functions.len(), 2);
    assert_eq!(prog.functions[0].name, "inc");
    assert_eq!(prog.functions[1].name, "main");
    assert_eq!(prog.body.len(), 2);
    assert!(matches!(prog.body[0], Stmt::Call { ref name, .. } if name == "inc"));
    assert!(matches!(prog.body[1], Stmt::Assign { ref target, .. } if target == "y"));
}

#[test]
fn test_parse_invalid_syntax_fails() {
    let result = parse_program_file("invalid_syntax.minic");
    assert!(result.is_err(), "invalid syntax should fail to parse");
}
