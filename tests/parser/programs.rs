use super::helpers::parse_program_file;

#[test]
fn parse_empty_program() {
    let prog = parse_program_file("empty.minic").expect("empty program should parse");
    assert!(prog.functions.is_empty());
}

#[test]
fn parse_main_only() {
    let prog = parse_program_file("statements_only.minic").expect("main-only program should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "main");
}

#[test]
fn parse_function_single() {
    let prog =
        parse_program_file("function_single.minic").expect("single-function program should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "foo");
}

#[test]
fn parse_function_with_block() {
    let prog =
        parse_program_file("function_with_block.minic").expect("function with block should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "add");
}

#[test]
fn parse_full_program() {
    let prog = parse_program_file("full_program.minic").expect("full program should parse");
    assert_eq!(prog.functions.len(), 2);
    assert_eq!(prog.functions[0].name, "inc");
    assert_eq!(prog.functions[1].name, "main");
}

#[test]
fn parse_invalid_syntax_fails() {
    assert!(parse_program_file("invalid_syntax.minic").is_err());
}

#[test]
fn parse_top_level_statements_fail() {
    assert!(parse_program_file("top_level_statements.minic").is_err());
}

#[test]
fn parse_program_with_user_defined_type_declarations() {
    let prog = parse_program_file("user_defined_types.minic")
        .expect("program with user-defined type declarations should parse");
    assert_eq!(prog.type_declarations.len(), 2);
    assert_eq!(prog.type_declarations[0].identifier, "Point");
    assert_eq!(prog.type_declarations[1].identifier, "Kind");
}
