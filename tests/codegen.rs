//! Integration tests for the Three-Address Code (TAC) generator.
//!
//! Each test parses and type-checks a small MiniC program, generates its TAC,
//! and asserts on the resulting instruction stream. The `assert` and `test`
//! cases exercise the Project 8 tie-in.

use nom::combinator::all_consuming;

use mini_c::codegen::tac_code_gen::generate_tac;
use mini_c::ir::ast::CheckedProgram;
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::parser::program;
use mini_c::semantic::type_check;

fn tac_of(src: &str) -> Vec<Instruction> {
    let (_, prog) = all_consuming(program)(src).expect("parse should succeed");
    let checked: CheckedProgram = type_check(&prog).expect("type check should succeed");
    generate_tac(&checked)
}

fn has_label(instrs: &[Instruction], label: &str) -> bool {
    instrs
        .iter()
        .any(|i| matches!(i, Instruction::Label(l) if l == label))
}

#[test]
fn arithmetic_assignment_uses_temp_and_copy() {
    // total = a + b  ->  temp := a + b ; total := temp
    let instrs = tac_of("void main() { int a = 1; int b = 2; int total = a + b; }");

    let has_add = instrs.iter().any(|i| {
        matches!(
            i,
            Instruction::BinaryAssignment(Operator::Add, Address::Temporary(_, _), _, _)
        )
    });
    assert!(has_add, "expected a binary Add into a temporary: {instrs:?}");

    // The declaration copies the temporary into `total`.
    let has_copy_total = instrs.iter().any(|i| {
        matches!(
            i,
            Instruction::CopyAssignment(Address::Variable(name, _), Address::Temporary(_, _))
                if name == "total"
        )
    });
    assert!(has_copy_total, "expected copy into `total`: {instrs:?}");
}

#[test]
fn if_else_emits_two_branches_and_join() {
    let instrs = tac_of(
        "int f(int x) { if x >= 0 { return 1; } else { return 0; } } void main() { print(f(1)); }",
    );
    // A relational conditional jump drives the branch selection.
    let has_rel = instrs.iter().any(|i| {
        matches!(
            i,
            Instruction::ConditionalJMPRelational(Operator::GTE, _, _, _)
        )
    });
    assert!(has_rel, "expected relational conditional jump: {instrs:?}");
    // Two returns, one per branch.
    let returns = instrs
        .iter()
        .filter(|i| matches!(i, Instruction::Return(_)))
        .count();
    assert_eq!(returns, 2, "expected one return per branch: {instrs:?}");
}

#[test]
fn while_loop_has_back_edge() {
    let instrs = tac_of(
        "void main() { int i = 0; while i < 3 { i = i + 1; } }",
    );
    // A while loop emits a back-edge: an unconditional jump to the loop head.
    let jumps = instrs
        .iter()
        .filter(|i| matches!(i, Instruction::JMP(_)))
        .count();
    assert!(jumps >= 1, "expected a back-edge jump: {instrs:?}");
    let has_rel = instrs.iter().any(|i| {
        matches!(i, Instruction::ConditionalJMPRelational(Operator::LT, _, _, _))
    });
    assert!(has_rel, "expected relational loop condition: {instrs:?}");
}

#[test]
fn return_with_value_emits_return_instruction() {
    let instrs = tac_of("int f() { return 42; } void main() { print(f()); }");
    let has_return_value = instrs
        .iter()
        .any(|i| matches!(i, Instruction::Return(Some(_))));
    assert!(has_return_value, "expected `return <addr>`: {instrs:?}");
}

#[test]
fn function_call_as_expression_binds_result_temp() {
    let instrs = tac_of("int f() { return 1; } void main() { int x = f(); }");
    // A call used as a value binds its result into a temporary.
    let has_valued_call = instrs
        .iter()
        .any(|i| matches!(i, Instruction::Call(Some(_), name, _) if name == "f"));
    assert!(has_valued_call, "expected `t := call f, 0`: {instrs:?}");
}

// ---- Project 8 tie-in -----------------------------------------------------

#[test]
fn assert_lowers_to_conditional_jump_and_failure_call() {
    let instrs = tac_of("test \"t\" { assert 1 == 1; }");

    // The assertion condition becomes a relational conditional jump.
    let has_rel = instrs.iter().any(|i| {
        matches!(i, Instruction::ConditionalJMPRelational(Operator::EQ, _, _, _))
    });
    assert!(has_rel, "expected relational jump for assert: {instrs:?}");

    // Failure path calls the `assert_fail` runtime routine with one argument.
    let has_fail_call = instrs
        .iter()
        .any(|i| matches!(i, Instruction::Call(None, name, 1) if name == "assert_fail"));
    assert!(has_fail_call, "expected `call assert_fail, 1`: {instrs:?}");
}

#[test]
fn each_test_block_becomes_a_labelled_routine() {
    let instrs = tac_of(
        "test \"first case\" { assert true; } test \"second\" { assert true; }",
    );
    assert!(
        has_label(&instrs, "test_first_case"),
        "expected label test_first_case: {instrs:?}"
    );
    assert!(
        has_label(&instrs, "test_second"),
        "expected label test_second: {instrs:?}"
    );
}
