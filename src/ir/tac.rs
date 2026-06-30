//! Representação do Three-Address Code (TAC) do MiniC.
//!
//! Um programa TAC é uma sequência linear de [`Instruction`]s. Cada instrução
//! opera sobre no máximo três endereços ([`Address`]): destino, operandos e,
//! em saltos, o label de destino.

use std::fmt;

// Tipos reutilizados da AST: nome de variável, literal e tipo MiniC.
use crate::ir::ast::{Literal, Name, Type};

/// Nome de label de salto (ex.: `"Label1:"`, `"main"`).
type Label = String;

/// Programa TAC completo: lista ordenada de instruções.
pub type TACProgram = Vec<Instruction>;

/// Endereço de um valor no TAC — onde ler ou gravar um dado.
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    /// Variável de programa declarada pelo usuário (ex.: `x`, `sum`).
    Variable(Name, Type),
    /// Valor literal imediato (ex.: `42`, `"hello"`, `true`).
    Constant(Literal, Type),
    /// Temporário gerado pelo compilador durante a tradução (ex.: `temp1`).
    Temporary(Name, Type),
}

/// Uma instrução TAC elementar.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Marca um ponto do código (início de função ou alvo de salto).
    Label(Label),
    /// Cópia simples: `dst = src`.
    CopyAssignment(Address, Address),
    /// Operação unária: `dst = op src` (ex.: negativo).
    UnaryAssignment(Operator, Address, Address),
    /// Operação binária: `dst = lhs op rhs` (ex.: `temp1 = x + y`).
    BinaryAssignment(Operator, Address, Address, Address),
    /// Salto incondicional para `label`.
    JMP(Label),
    /// Salta para `label` se `addr` for verdadeiro (truthy).
    ConditionalJMP(Address, Label),
    /// Salta para `label` se `addr` for falso (falsy).
    ConditionalJMPFalse(Address, Label),
    /// Salta para `label` se `lhs op rhs` for verdadeiro (ex.: `x < y`).
    ConditionalJMPRelational(Operator, Address, Address, Label),
    /// Passa um argumento antes de uma chamada de função.
    Param(Address),
    /// Chamada de função: `call name, n` ou `dest = call name, n`.
    /// `None` = retorno descartado; `Some(dest)` = guarda o retorno em `dest`.
    Call(Option<Address>, Name, usize),
    /// Escrita em array: `base[index] = value`.
    Store(Address, Address, Address),
    /// Leitura de array: `dest = base[index]`.
    Load(Address, Address, Address),
    /// Retorno de função: `return` (void) ou `return addr`.
    Return(Option<Address>),
}

/// Operadores usados em instruções aritméticas, unárias e comparações.
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

/// Formata um endereço para exibição textual (usado pela flag `--tac`).
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Variáveis e temporários: imprime só o nome (tipo é metadado interno).
            Address::Variable(name, _) | Address::Temporary(name, _) => write!(f, "{name}"),
            // Constantes: delega para `Display` de `Literal`.
            Address::Constant(lit, _) => write!(f, "{lit}"),
        }
    }
}

/// Formata literais MiniC como apareceriam no código fonte.
impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{n}"),
            Literal::Float(x) => write!(f, "{x}"),
            Literal::Str(s) => write!(f, "\"{s}\""), // strings entre aspas
            Literal::Bool(b) => write!(f, "{b}"),
        }
    }
}

/// Converte operador interno para símbolo infixo legível (`+`, `<=`, etc.).
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

/// Formata cada variante de instrução TAC como pseudo-código legível.
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

// Exemplo comentado de TAC equivalente a: do i = i + 1 while(a[i] < v);
//
// L1:                                    # Label("L1")
//   t1 = i + 1           # BinaryAssignment(Add, t1, i, 1)
//   i  = t1              # CopyAssignment(i, t1)
//   t2 = i * 8           # BinaryAssignment(Mul, t2, i, 8)
//   t3 = a[t2]           # Load(t3, a, t2)
//   if t3 < v goto L1    # ConditionalJMPRelational(LT, t3, v, "L1")
//
