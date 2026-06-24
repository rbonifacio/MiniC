use crate::ir::ast::{Literal, Name, Type};
use std::fmt;

type Label = String;

pub type TACProgram = Vec<Instruction>;

#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Variable(Name, Type),
    Constant(Literal, Type),
    Temporary(Name, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Label(Label),
    CopyAssignment(Address, Address),
    UnaryAssignment(Operator, Address, Address),
    BinaryAssignment(Operator, Address, Address, Address),
    JMP(Label),
    ConditionalJMP(Address, Label),
    ConditionalJMPFalse(Address, Label),
    ConditionalJMPRelational(Operator, Address, Address, Label),
    Param(Address),
    Call(Option<Address>, Name, usize), // It is either 'call p, n' or 'y = call p, n'
    Store(Address, Address, Address),   // x[i] = y
    Load(Address, Address, Address),    // x = y[i]
    Return(Option<Address>),
}

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
    SL,  // shift left
    SR,  // shift right
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Variable(name, _) | Address::Temporary(name, _) => write!(f, "{}", name),
            Address::Constant(literal, _) => write!(f, "{}", literal),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(x) => write!(f, "{}", x),
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
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
        write!(f, "{}", op)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Label(label) => {
                if label.ends_with(':') {
                    write!(f, "{}", label)
                } else {
                    write!(f, "{}:", label)
                }
            }
            Instruction::CopyAssignment(dst, src) => write!(f, "{} = {}", dst, src),
            Instruction::UnaryAssignment(op, dst, src) => write!(f, "{} = {}{}", dst, op, src),
            Instruction::BinaryAssignment(op, dst, left, right) => {
                write!(f, "{} = {} {} {}", dst, left, op, right)
            }
            Instruction::JMP(label) => write!(f, "goto {}", label),
            Instruction::ConditionalJMP(cond, label) => write!(f, "if {} goto {}", cond, label),
            Instruction::ConditionalJMPFalse(cond, label) => {
                write!(f, "if_false {} goto {}", cond, label)
            }
            Instruction::ConditionalJMPRelational(op, left, right, label) => {
                write!(f, "if {} {} {} goto {}", left, op, right, label)
            }
            Instruction::Param(addr) => write!(f, "param {}", addr),
            Instruction::Call(None, name, arity) => write!(f, "call {}, {}", name, arity),
            Instruction::Call(Some(dst), name, arity) => {
                write!(f, "{} = call {}, {}", dst, name, arity)
            }
            Instruction::Store(base, index, value) => write!(f, "{}[{}] = {}", base, index, value),
            Instruction::Load(dst, base, index) => write!(f, "{} = {}[{}]", dst, base, index),
            Instruction::Return(None) => write!(f, "return"),
            Instruction::Return(Some(addr)) => write!(f, "return {}", addr),
        }
    }
}

// do i = i + 1 while(a[i] < v);
//
// L1:                                    # Label("L1")
//   t1 = i + 1           # BinaryAssignment(Add, t1, i, 1)
//   i  = t1              # CopyAssignment(i, t1)
//   t2 = i * 8           # BinaryAssignment(Mul, t2, i, 8)
//   t3 = a[t2]           # Load(t3, a, t2)
//   if t3 < v goto L1    # ConditionalJMPRelational(LT, t3, v, "L1")
//
