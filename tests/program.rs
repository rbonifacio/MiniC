//! Integration tests for parsing MiniC programs from files.

use nom::combinator::all_consuming;
use std::path::Path;
use mini_c::ir::ast::{Statement, Type, UncheckedProgram};
use mini_c::parser::program;

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
    let prog =
        parse_program_file("statements_only.minic").expect("main-only program should parse");
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
    assert!(
        matches!(prog.functions[0].body.stmt, Statement::Decl { ref name, .. } if name == "x")
    );
}

#[test]
fn test_parse_function_with_block() {
    let prog =
        parse_program_file("function_with_block.minic").expect("function with block should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "add");
    assert_eq!(
        prog.functions[0].params,
        vec![("x".to_string(), Type::Int), ("y".to_string(), Type::Int)]
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
    assert!(result.is_err(), "top-level statements without def should fail to parse");
}

#[test]
fn test_parse_pointer_feature_program() {
    let prog = parse_program_file("pointer_feature.minic").expect("pointer feature should parse");
    assert_eq!(prog.functions.len(), 2);
    assert_eq!(prog.functions[0].name, "increment");
    assert_eq!(prog.functions[1].name, "main");
    assert_eq!(prog.functions[0].params, vec![("p".to_string(), Type::Pointer(Box::new(Type::Int)))]);

    if let Statement::Block { ref seq } = prog.functions[1].body.stmt {
        assert_eq!(seq.len(), 4);
        assert!(matches!(seq[0].stmt, Statement::Decl { ref name, .. } if name == "x"));
        assert!(matches!(seq[1].stmt, Statement::Decl { ref name, .. } if name == "y"));
        assert!(matches!(seq[2].stmt, Statement::Call { ref name, .. } if name == "increment"));
        assert!(matches!(seq[3].stmt, Statement::Call { ref name, .. } if name == "print"));
    } else {
        panic!("expected main to have block body");
    }
}

#[test]
fn test_parse_pointer_function_program() {
    let prog = parse_program_file("pointer_function.minic").expect("pointer function should parse");
    assert_eq!(prog.functions.len(), 2);
    assert_eq!(prog.functions[0].name, "changeRef");
    assert_eq!(prog.functions[1].name, "main");
    assert_eq!(
        prog.functions[0].params,
        vec![
            ("x".to_string(), Type::Pointer(Box::new(Type::Int))),
            ("y".to_string(), Type::Pointer(Box::new(Type::Int)))
        ]
    );

    if let Statement::Block { ref seq } = prog.functions[0].body.stmt {
        assert_eq!(seq.len(), 2);
        assert!(matches!(seq[0].stmt, Statement::Assign { .. }));
        assert!(matches!(seq[1].stmt, Statement::Return(_)));
    } else {
        panic!("expected changeRef to have block body");
    }

    if let Statement::Block { ref seq } = prog.functions[1].body.stmt {
        assert_eq!(seq.len(), 5);
        assert!(matches!(seq[0].stmt, Statement::Decl { ref name, .. } if name == "x"));
        assert!(matches!(seq[1].stmt, Statement::Decl { ref name, .. } if name == "y"));
        assert!(matches!(seq[2].stmt, Statement::Decl { ref name, .. } if name == "x_ref"));
        assert!(matches!(seq[3].stmt, Statement::Decl { ref name, .. } if name == "y_ref"));
        assert!(matches!(seq[4].stmt, Statement::Call { ref name, .. } if name == "changeRef"));
    } else {
        panic!("expected main to have block body");
    }
}

#[test]
fn test_parse_pointer_init_program() {
    let prog = parse_program_file("pointer_init.minic").expect("pointer init should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "main");

    if let Statement::Block { ref seq } = prog.functions[0].body.stmt {
        assert_eq!(seq.len(), 8);
        assert!(matches!(seq[0].stmt, Statement::Decl { ref name, .. } if name == "a"));
        assert!(matches!(seq[1].stmt, Statement::Decl { ref name, .. } if name == "b"));
        assert!(matches!(seq[2].stmt, Statement::Decl { ref name, .. } if name == "c"));
        assert!(matches!(seq[3].stmt, Statement::Decl { ref name, .. } if name == "d"));
        assert!(matches!(seq[4].stmt, Statement::Decl { ref name, .. } if name == "a_ref"));
        assert!(matches!(seq[5].stmt, Statement::Decl { ref name, .. } if name == "b_ref"));
        assert!(matches!(seq[6].stmt, Statement::Decl { ref name, .. } if name == "c_ref"));
        assert!(matches!(seq[7].stmt, Statement::Decl { ref name, .. } if name == "d_ref"));
    } else {
        panic!("expected main to have block body");
    }
}