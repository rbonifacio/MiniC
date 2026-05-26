use crate::ir::ast::{Literal, Name, Type};

type Label = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Variable(Name, Type),
    Constant(Literal, Type),
    Temporary(Name, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Label(Label),
    CopyAssignment(Address, Address),
    UnaryAssignment(Operator, Address, Address),
    BinaryAssignment(Operator, Address, Address, Address),
    JMP(Label),
    ConditionalJMP(Address, Label),
    ConditionalJMPFalse(Address, Label),
    ConditionalJMPRelational(Operator, Address, Address, Label),
    Param(Address),
    Call(Option<Address>, Name, usize),
    Store(Address, Address, Address),
    Load(Address, Address, Address),
    Return(Option<Address>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,               // a + b
    Sub,               // a - b
    Mul,               // a * b
    Div,               // a / b
    Neg,               // -a
    LT,                // a < b
    LTE,               // a <= b
    GT,                // a > b
    GTE,               // a >= b
    EQ,                // a == b
    NE,                // a != b
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
