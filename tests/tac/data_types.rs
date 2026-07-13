use mini_c::codegen::tac_code_gen::{translate_program, Environment};
use mini_c::ir::ast::{CheckedProgram, Literal, Type, UncheckedProgram};
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::parser::program;
use mini_c::semantic::type_check;
use nom::combinator::all_consuming;
use std::path::Path;

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn parse_fixture(name: &str) -> UncheckedProgram {
    let source = std::fs::read_to_string(fixtures_dir().join(name)).expect("fixture should exist");
    let result = all_consuming(program)(source.trim())
        .map_err(|e| e.map_input(String::from))
        .expect("fixture should parse");
    result.1
}

fn type_check_fixture(name: &str) -> Result<CheckedProgram, mini_c::semantic::TypeError> {
    type_check(&parse_fixture(name))
}

fn type_check_src(src: &str) -> CheckedProgram {
    let (_, prog) = all_consuming(program)(src).expect("source should parse");
    type_check(&prog).expect("source should type-check")
}

#[test]
fn struct_init_tac() {
    let checked = type_check_fixture("tac_struct.minic").expect("struct fixture should type-check");
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::Label("main".to_string()),
            Instruction::CopyAssignment(
                Address::Variable("p.valid".to_string(), Type::Bool),
                Address::Constant(Literal::Bool(true), Type::Bool)
            ),
            Instruction::CopyAssignment(
                Address::Variable("p.x".to_string(), Type::Int),
                Address::Constant(Literal::Int(42), Type::Int)
            ),
            Instruction::Param(Address::Variable("p.valid".to_string(), Type::Bool)),
            Instruction::Call(None, "print".to_string(), 1),
        ]
    );
}

#[test]
fn enum_init_match_tac() {
    let checked = type_check_fixture("tac_enum.minic").expect("enum fixture should type-check");
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::Label("main".to_string()),
            Instruction::CopyAssignment(
                Address::Variable("k.tag".to_string(), Type::Int),
                Address::Constant(Literal::Int(0), Type::Int)
            ),
            Instruction::CopyAssignment(
                Address::Variable("k.payload".to_string(), Type::Int),
                Address::Constant(Literal::Int(42), Type::Int)
            ),
            Instruction::ConditionalJMPRelational(
                Operator::NE,
                Address::Variable("k.tag".to_string(), Type::Int),
                Address::Constant(Literal::Int(0), Type::Int),
                "Label2:".to_string()
            ),
            Instruction::CopyAssignment(
                Address::Temporary("_match1".to_string(), Type::Int),
                Address::Variable("k.payload".to_string(), Type::Int)
            ),
            Instruction::Param(Address::Temporary("_match1".to_string(), Type::Int)),
            Instruction::Call(None, "print".to_string(), 1),
            Instruction::JMP("Label1:".to_string()),
            Instruction::Label("Label2:".to_string()),
            Instruction::ConditionalJMPRelational(
                Operator::NE,
                Address::Variable("k.tag".to_string(), Type::Int),
                Address::Constant(Literal::Int(1), Type::Int),
                "Label3:".to_string()
            ),
            Instruction::Param(Address::Constant(Literal::Int(0), Type::Int)),
            Instruction::Call(None, "print".to_string(), 1),
            Instruction::JMP("Label1:".to_string()),
            Instruction::Label("Label3:".to_string()),
            Instruction::Label("Label1:".to_string()),
        ]
    );
}

#[test]
fn nested_types_tac() {
    let checked = type_check_fixture("tac_nested.minic").expect("nested fixture should type-check");
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::Label("main".to_string()),
            Instruction::CopyAssignment(
                Address::Variable("o.field.tag".to_string(), Type::Int),
                Address::Constant(Literal::Int(0), Type::Int)
            ),
            Instruction::CopyAssignment(
                Address::Variable("o.field.payload".to_string(), Type::Int),
                Address::Constant(Literal::Int(42), Type::Int)
            ),
        ]
    );
}

#[test]
fn struct_copy_tac() {
    let checked = type_check_src(
        "struct Point { int x; int y; } void main() { struct Point p = { .x = 1, .y = 2 }; struct Point q = p; print(q.x); }",
    );
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::Label("main".to_string()),
            Instruction::CopyAssignment(
                Address::Variable("p.x".to_string(), Type::Int),
                Address::Constant(Literal::Int(1), Type::Int)
            ),
            Instruction::CopyAssignment(
                Address::Variable("p.y".to_string(), Type::Int),
                Address::Constant(Literal::Int(2), Type::Int)
            ),
            Instruction::CopyAssignment(
                Address::Variable("q.x".to_string(), Type::Int),
                Address::Variable("p.x".to_string(), Type::Int)
            ),
            Instruction::CopyAssignment(
                Address::Variable("q.y".to_string(), Type::Int),
                Address::Variable("p.y".to_string(), Type::Int)
            ),
            Instruction::Param(Address::Variable("q.x".to_string(), Type::Int)),
            Instruction::Call(None, "print".to_string(), 1),
        ]
    );
}

#[test]
fn struct_assignment_tac() {
    let checked = type_check_src(
        "struct Point { int x; int y; } void main() { struct Point p = { .x = 1, .y = 2 }; struct Point q = { .x = 0, .y = 0 }; q = p; print(q.y); }",
    );
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("q.x".to_string(), Type::Int),
        Address::Variable("p.x".to_string(), Type::Int)
    )));
    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("q.y".to_string(), Type::Int),
        Address::Variable("p.y".to_string(), Type::Int)
    )));
    assert!(!instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("q".to_string(), Type::Struct("Point".to_string())),
        Address::Variable("p".to_string(), Type::Struct("Point".to_string()))
    )));
}

#[test]
fn enum_copy_and_match_tac() {
    let checked = type_check_src(
        "enum Option { int Some; None; } void main() { enum Option a = { .Some = 10 }; enum Option b = a; match b { case Some: { print(Some); } case None: { print(0); } } }",
    );
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("b.tag".to_string(), Type::Int),
        Address::Variable("a.tag".to_string(), Type::Int)
    )));
    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("b.payload".to_string(), Type::Int),
        Address::Variable("a.payload".to_string(), Type::Int)
    )));
    assert!(instructions.contains(&Instruction::ConditionalJMPRelational(
        Operator::NE,
        Address::Variable("b.tag".to_string(), Type::Int),
        Address::Constant(Literal::Int(1), Type::Int),
        "Label3:".to_string()
    )));
}

#[test]
fn enum_assignment_tac() {
    let checked = type_check_src(
        "enum Option { int Some; None; } void main() { enum Option a = { .Some = 10 }; enum Option b = { .None }; b = a; match b { case Some: { print(Some); } case None: { print(0); } } }",
    );
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("b.tag".to_string(), Type::Int),
        Address::Variable("a.tag".to_string(), Type::Int)
    )));
    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("b.payload".to_string(), Type::Int),
        Address::Variable("a.payload".to_string(), Type::Int)
    )));
}

#[test]
fn nested_struct_copy_tac() {
    let checked = type_check_src(
        "struct Inner { int x; } struct Outer { struct Inner inner; } void main() { struct Outer a = { .inner = { .x = 7 } }; struct Outer b = a; print(b.inner.x); }",
    );
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("b.inner.x".to_string(), Type::Int),
        Address::Variable("a.inner.x".to_string(), Type::Int)
    )));
}

#[test]
fn match_payload_binding_uses_internal_tac_temp() {
    let checked = type_check_src(
        "enum Option { int Some; None; } void main() { int Some = 5; enum Option a = { .Some = 10 }; match a { case Some: { print(Some); } case None: { print(0); } } print(Some); }",
    );
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert!(instructions.contains(&Instruction::CopyAssignment(
        Address::Temporary("_match1".to_string(), Type::Int),
        Address::Variable("a.payload".to_string(), Type::Int)
    )));
    assert!(instructions.contains(&Instruction::Param(Address::Temporary(
        "_match1".to_string(),
        Type::Int
    ))));
    assert!(instructions.contains(&Instruction::Param(Address::Variable(
        "Some".to_string(),
        Type::Int
    ))));
    assert!(!instructions.contains(&Instruction::CopyAssignment(
        Address::Variable("Some".to_string(), Type::Int),
        Address::Variable("a.payload".to_string(), Type::Int)
    )));
}
