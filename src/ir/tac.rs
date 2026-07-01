//! MiniC Three-Address Code (TAC) representation.
//!
//! A TAC program is a linear sequence of [`Instruction`]s. Each instruction
//! operates on at most three addresses ([`Address`]): destination, operands, and,
//! for jumps, the target label.

use std::fmt;

// Types reused from the AST: variable name, literal, and MiniC type.
use crate::ir::ast::{Literal, Name, Type};

/// Jump label name (e.g. `"Label1:"`, `"main"`).
type Label = String;

/// Complete TAC program: ordered list of instructions.
pub type TACProgram = Vec<Instruction>;

/// Address of a value in TAC — where to read or write data.
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    /// User-declared program variable (e.g. `x`, `sum`).
    Variable(Name, Type),
    /// Immediate literal value (e.g. `42`, `"hello"`, `true`).
    Constant(Literal, Type),
    /// Compiler-generated temporary during translation (e.g. `temp1`).
    Temporary(Name, Type),
}

/// A single TAC instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Marks a code location (function entry or jump target).
    Label(Label),
    /// Simple copy: `dst = src`.
    CopyAssignment(Address, Address),
    /// Unary operation: `dst = op src` (e.g. negation).
    UnaryAssignment(Operator, Address, Address),
    /// Binary operation: `dst = lhs op rhs` (e.g. `temp1 = x + y`).
    BinaryAssignment(Operator, Address, Address, Address),
    /// Unconditional jump to `label`.
    JMP(Label),
    /// Jump to `label` if `addr` is true (truthy).
    ConditionalJMP(Address, Label),
    /// Jump to `label` if `addr` is false (falsy).
    ConditionalJMPFalse(Address, Label),
    /// Jump to `label` if `lhs op rhs` is true (e.g. `x < y`).
    ConditionalJMPRelational(Operator, Address, Address, Label),
    /// Pass an argument before a function call.
    Param(Address),
    /// Function call: `call name, n` or `dest = call name, n`.
    /// `None` = discard return value; `Some(dest)` = store return in `dest`.
    Call(Option<Address>, Name, usize),
    /// Array write: `base[index] = value`.
    Store(Address, Address, Address),
    /// Array read: `dest = base[index]`.
    Load(Address, Address, Address),
    /// Function return: `return` (void) or `return addr`.
    Return(Option<Address>),
}

/// Operators used in arithmetic, unary, and comparison instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add, // a + b
    Sub, // a - b
    Mul, // a * b
    Div, // a / b
    Neg, // -a
    LT,  // a < b
    LTE, // a <= b
    GT,  // a > b
    GTE, // a >= b
    EQ,  // a == b
    NE,  // a != b
    SL,  // shift left (a << b)
    SR,  // shift right (a >> b)
}

/// Formats an address for textual display (used by the `--tac` flag).
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Variables and temporaries: print name only (type is internal metadata).
            Address::Variable(name, _) | Address::Temporary(name, _) => write!(f, "{name}"),
            // Constants: delegate to `Literal`'s `Display`.
            Address::Constant(lit, _) => write!(f, "{lit}"),
        }
    }
}

/// Formats MiniC literals as they would appear in source code.
impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{n}"),
            Literal::Float(x) => write!(f, "{x}"),
            Literal::Str(s) => write!(f, "\"{s}\""), // strings in quotes
            Literal::Bool(b) => write!(f, "{b}"),
        }
    }
}

/// Converts internal operator to readable infix symbol (`+`, `<=`, etc.).
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
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
        };
        write!(f, "{s}")
    }
}

/// Formats each TAC instruction variant as readable pseudo-code.
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Label(label) => write!(f, "{label}"),
            Instruction::CopyAssignment(dst, src) => write!(f, "{dst} = {src}"),
            Instruction::UnaryAssignment(op, dst, src) => write!(f, "{dst} = {op}{src}"),
            Instruction::BinaryAssignment(op, dst, lhs, rhs) => {
                write!(f, "{dst} = {lhs} {op} {rhs}")
            }
            Instruction::JMP(label) => write!(f, "goto {label}"),
            Instruction::ConditionalJMP(addr, label) => write!(f, "if {addr} goto {label}"),
            Instruction::ConditionalJMPFalse(addr, label) => {
                write!(f, "if !{addr} goto {label}")
            }
            Instruction::ConditionalJMPRelational(op, lhs, rhs, label) => {
                write!(f, "if {lhs} {op} {rhs} goto {label}")
            }
            Instruction::Param(addr) => write!(f, "param {addr}"),
            Instruction::Call(None, name, n) => write!(f, "call {name}, {n}"),
            Instruction::Call(Some(dest), name, n) => write!(f, "{dest} = call {name}, {n}"),
            Instruction::Store(base, index, value) => write!(f, "{base}[{index}] = {value}"),
            Instruction::Load(dest, base, index) => write!(f, "{dest} = {base}[{index}]"),
            Instruction::Return(None) => write!(f, "return"),
            Instruction::Return(Some(addr)) => write!(f, "return {addr}"),
        }
    }
}

// Commented TAC example equivalent to: do i = i + 1 while(a[i] < v);
//
// L1:                                    # Label("L1")
//   t1 = i + 1           # BinaryAssignment(Add, t1, i, 1)
//   i  = t1              # CopyAssignment(i, t1)
//   t2 = i * 8           # BinaryAssignment(Mul, t2, i, 8)
//   t3 = a[t2]           # Load(t3, a, t2)
//   if t3 < v goto L1    # ConditionalJMPRelational(LT, t3, v, "L1")
//
