//! Integration tests for parsing MiniC programs from files.

use mini_c::ir::ast::{IdentifierDecl, Statement, Type, UncheckedProgram};
use mini_c::parser::program;
use nom::combinator::all_consuming;
use std::path::Path;

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn parse_program_file(name: &str) -> Result<UncheckedProgram, nom::Err<nom::error::Error<String>>> {
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
}

#[test]
fn test_parse_main_only() {
    let prog = parse_program_file("statements_only.minic").expect("main-only program should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "main");
    assert!(matches!(prog.functions[0].body.stmt, Statement::Block { ref seq } if seq.len() == 2));
    if let Statement::Block { ref seq } = prog.functions[0].body.stmt {
        assert!(matches!(seq[0].stmt, Statement::Decl { ref name, .. } if name == "x"));
        assert!(matches!(seq[1].stmt, Statement::Decl { ref name, .. } if name == "y"));
    }
}

#[test]
fn test_parse_function_single() {
    let prog =
        parse_program_file("function_single.minic").expect("single-function program should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "foo");
    assert!(prog.functions[0].params.is_empty());
    assert!(matches!(prog.functions[0].body.stmt, Statement::Decl { ref name, .. } if name == "x"));
}

#[test]
fn test_parse_function_with_block() {
    let prog =
        parse_program_file("function_with_block.minic").expect("function with block should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "add");
    assert_eq!(
        prog.functions[0].params,
        vec![
            IdentifierDecl {
                name: "x".to_string(),
                ty: Type::Int,
            },
            IdentifierDecl {
                name: "y".to_string(),
                ty: Type::Int,
            },
        ]
    );
    assert!(matches!(prog.functions[0].body.stmt, Statement::Block { ref seq } if seq.len() == 2));
}

#[test]
fn test_parse_full_program() {
    let prog = parse_program_file("full_program.minic").expect("full program should parse");
    assert_eq!(prog.functions.len(), 2);
    assert_eq!(prog.functions[0].name, "inc");
    assert_eq!(prog.functions[1].name, "main");
    let main_body = &prog.functions[1].body.stmt;
    if let Statement::Block { ref seq } = main_body {
        assert_eq!(seq.len(), 2);
        assert!(matches!(seq[0].stmt, Statement::Call { ref name, .. } if name == "inc"));
        assert!(matches!(seq[1].stmt, Statement::Decl { ref name, .. } if name == "y"));
    } else {
        panic!("expected main to have block body");
    }
}

#[test]
fn test_parse_invalid_syntax_fails() {
    let result = parse_program_file("invalid_syntax.minic");
    assert!(result.is_err(), "invalid syntax should fail to parse");
}

#[test]
fn test_parse_top_level_statements_fail() {
    let result = parse_program_file("top_level_statements.minic");
    assert!(
        result.is_err(),
        "top-level statements without def should fail to parse"
    );
}

#[test]
fn test_parse_program_with_tagged_declarations() {
    let prog = parse_program_file("tagged_types.minic")
        .expect("program with tagged type declarations should parse");
    assert_eq!(prog.tagged_types.len(), 3);
    assert_eq!(prog.tagged_types[0].tag_name, "Point");
    assert_eq!(prog.tagged_types[1].tag_name, "Payload");
    assert_eq!(prog.tagged_types[2].tag_name, "Kind");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "main");
}
