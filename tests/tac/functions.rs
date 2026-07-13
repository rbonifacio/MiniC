use super::helpers::type_check_fixture;
use mini_c::codegen::tac_code_gen::{translate_program, Environment};
use mini_c::ir::ast::Type;
use mini_c::ir::tac::{Address, Instruction};

#[test]
fn print_call_tac() {
    let checked = type_check_fixture("tac_struct.minic").expect("fixture should type-check");
    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    assert_eq!(
        instructions[3..],
        [
            Instruction::Param(Address::Variable("p.valid".to_string(), Type::Bool)),
            Instruction::Call(None, "print".to_string(), 1),
        ]
    );
}
