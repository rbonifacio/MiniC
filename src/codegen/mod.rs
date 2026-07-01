//! Code generation.
//!
//! This module lowers a type-checked MiniC program into a lower-level
//! representation suitable for later back-end stages. The only target
//! currently implemented is [`Three-Address Code`](crate::ir::tac): a flat
//! list of simple instructions in which every operation refers to at most
//! three addresses.
//!
//! * [`tac_code_gen::generate_tac`] produces the TAC for a whole program.
//! * [`format_program`] renders a TAC program as human-readable text, used by
//!   the `--tac` CLI mode and by the tests.

pub mod tac_code_gen;

use crate::ir::ast::Literal;
use crate::ir::tac::{Address, Instruction, Operator, TACProgram};

/// Render a TAC program as human-readable text, one instruction per line.
///
/// Labels are printed flush-left with a trailing colon; every other
/// instruction is indented two spaces. The output mirrors the notation used in
/// the translation-rule slides (`t1 := a + b`, `if x >= y goto L3`, …).
pub fn format_program(program: &TACProgram) -> String {
    program
        .iter()
        .map(format_instruction)
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Label(label) => format!("{}:", label),
        Instruction::CopyAssignment(dst, src) => {
            format!("  {} := {}", format_address(dst), format_address(src))
        }
        Instruction::UnaryAssignment(op, dst, a) => format!(
            "  {} := {} {}",
            format_address(dst),
            format_operator(op),
            format_address(a)
        ),
        Instruction::BinaryAssignment(op, dst, a, b) => format!(
            "  {} := {} {} {}",
            format_address(dst),
            format_address(a),
            format_operator(op),
            format_address(b)
        ),
        Instruction::JMP(label) => format!("  goto {}", label),
        Instruction::ConditionalJMP(a, label) => {
            format!("  if {} goto {}", format_address(a), label)
        }
        Instruction::ConditionalJMPFalse(a, label) => {
            format!("  if_false {} goto {}", format_address(a), label)
        }
        Instruction::ConditionalJMPRelational(op, a, b, label) => format!(
            "  if {} {} {} goto {}",
            format_address(a),
            format_operator(op),
            format_address(b),
            label
        ),
        Instruction::Param(a) => format!("  param {}", format_address(a)),
        Instruction::Call(None, name, n) => format!("  call {}, {}", name, n),
        Instruction::Call(Some(dst), name, n) => {
            format!("  {} := call {}, {}", format_address(dst), name, n)
        }
        Instruction::Store(base, index, value) => format!(
            "  {}[{}] := {}",
            format_address(base),
            format_address(index),
            format_address(value)
        ),
        Instruction::Load(dst, base, index) => format!(
            "  {} := {}[{}]",
            format_address(dst),
            format_address(base),
            format_address(index)
        ),
        Instruction::Return(None) => "  return".to_string(),
        Instruction::Return(Some(a)) => format!("  return {}", format_address(a)),
    }
}

fn format_address(address: &Address) -> String {
    match address {
        Address::Variable(name, _) => name.clone(),
        Address::Temporary(name, _) => name.clone(),
        Address::Constant(literal, _) => format_literal(literal),
    }
}

fn format_literal(literal: &Literal) -> String {
    match literal {
        Literal::Int(n) => n.to_string(),
        Literal::Float(f) => f.to_string(),
        Literal::Bool(b) => b.to_string(),
        Literal::Str(s) => format!("{:?}", s),
    }
}

fn format_operator(op: &Operator) -> &'static str {
    match op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mul => "*",
        Operator::Div => "/",
        Operator::Neg => "-",
        Operator::LT => "<",
        Operator::LTE => "<=",
        Operator::GT => ">",
        Operator::GTE => ">=",
        Operator::EQ => "==",
        Operator::NE => "!=",
        Operator::SL => "<<",
        Operator::SR => ">>",
    }
}
