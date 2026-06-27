use crate::ir::ast::{Literal, Name, Type};
use std::fmt;

type Label = String;

pub type TACProgram = Vec<Instruction>;

#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Variable(Name, Type),
    Constant(Literal, Type),
    Temporary(Name, Type),
    FunctionLabel(Name),
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
    Call(Option<Address>, Name, usize),   // It is either 'call p, n' or 'y = call p, n'
    CallIndirect(Option<Address>, Address, usize),
    Store(Address, Address, Address),     // x[i] = y
    Load(Address, Address, Address),      // x = y[i]
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
    SL,                // shift left
    SR,                // shift right
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

// --- Display Formatter (TAC Printer) ---

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "void"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Array(ty) => write!(f, "{}[]", ty),
            Type::Fun(params, ret) => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Any => write!(f, "any"),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(v) => write!(f, "{}", v),
            Literal::Float(v) => write!(f, "{}", v),
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
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
        write!(f, "{}", op_str)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Variable(name, _) => write!(f, "{}", name),
            Address::Constant(lit, _) => write!(f, "{}", lit),
            Address::Temporary(name, _) => write!(f, "{}", name),
            Address::FunctionLabel(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Label(label) => {
                if label.ends_with(':') {
                    write!(f, "{}", label)
                } else {
                    write!(f, "Label {}", label)
                }
            }
            Instruction::CopyAssignment(dst, src) => {
                write!(f, "{} = {}", dst, src)
            }
            Instruction::UnaryAssignment(op, dst, src) => {
                write!(f, "{} = {} {}", dst, op, src)
            }
            Instruction::BinaryAssignment(op, dst, src1, src2) => {
                write!(f, "{} = {} {} {}", dst, src1, op, src2)
            }
            Instruction::JMP(label) => {
                write!(f, "jmp {}", label)
            }
            Instruction::ConditionalJMP(addr, label) => {
                write!(f, "if {} goto {}", addr, label)
            }
            Instruction::ConditionalJMPFalse(addr, label) => {
                write!(f, "ifFalse {} goto {}", addr, label)
            }
            Instruction::ConditionalJMPRelational(op, src1, src2, label) => {
                write!(f, "if {} {} {} goto {}", src1, op, src2, label)
            }
            Instruction::Param(addr) => {
                write!(f, "param {}", addr)
            }
            Instruction::Call(dst, name, arity) => {
                if let Some(d) = dst {
                    write!(f, "{} = call {}, {}", d, name, arity)
                } else {
                    write!(f, "call {}, {}", name, arity)
                }
            }
            Instruction::CallIndirect(dst, callee, arity) => {
                if let Some(d) = dst {
                    write!(f, "{} = call_indirect {}, {}", d, callee, arity)
                } else {
                    write!(f, "call_indirect {}, {}", callee, arity)
                }
            }
            Instruction::Store(arr, idx, val) => {
                write!(f, "{}[{}] = {}", arr, idx, val)
            }
            Instruction::Load(dst, arr, idx) => {
                write!(f, "{} = {}[{}]", dst, arr, idx)
            }
            Instruction::Return(val) => {
                if let Some(v) = val {
                    write!(f, "return {}", v)
                } else {
                    write!(f, "return")
                }
            }
        }
    }
}

// --- Hex Helpers for String Serialization ---

fn hex_encode(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<String, String> {
    if s.len() % 2 != 0 {
        return Err("Invalid hex length".to_string());
    }
    let mut bytes = Vec::new();
    for i in (0..s.len()).step_by(2) {
        let b = u8::from_str_radix(&s[i..i+2], 16)
            .map_err(|e| format!("Invalid hex byte: {}", e))?;
        bytes.push(b);
    }
    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in hex: {}", e))
}

// --- Serialization & Deserialization (Line-based) ---

fn serialize_type(ty: &Type) -> String {
    match ty {
        Type::Unit => "U".to_string(),
        Type::Int => "I".to_string(),
        Type::Float => "F".to_string(),
        Type::Bool => "B".to_string(),
        Type::Str => "S".to_string(),
        Type::Any => "A".to_string(),
        Type::Array(inner) => format!("[{}", serialize_type(inner)),
        Type::Fun(params, ret) => {
            let p_strs: Vec<String> = params.iter().map(|p| serialize_type(p)).collect();
            format!("(/{}/{})", p_strs.join(","), serialize_type(ret))
        }
    }
}

fn deserialize_type(s: &str) -> Result<(Type, &str), String> {
    if s.is_empty() {
        return Err("Empty type string".to_string());
    }
    let first = &s[0..1];
    let rest = &s[1..];
    match first {
        "U" => Ok((Type::Unit, rest)),
        "I" => Ok((Type::Int, rest)),
        "F" => Ok((Type::Float, rest)),
        "B" => Ok((Type::Bool, rest)),
        "S" => Ok((Type::Str, rest)),
        "A" => Ok((Type::Any, rest)),
        "[" => {
            let (inner, rest2) = deserialize_type(rest)?;
            Ok((Type::Array(Box::new(inner)), rest2))
        }
        "(" => {
            if !rest.starts_with('/') {
                return Err("Expected / after (".to_string());
            }
            let mut current = &rest[1..];
            let mut params = Vec::new();
            if current.starts_with('/') {
                current = &current[1..];
            } else {
                loop {
                    let (p_ty, next) = deserialize_type(current)?;
                    params.push(p_ty);
                    if next.starts_with(',') {
                        current = &next[1..];
                    } else if next.starts_with('/') {
                        current = &next[1..];
                        break;
                    } else {
                        return Err("Expected , or / in function type parameters".to_string());
                    }
                }
            }
            let (ret_ty, next) = deserialize_type(current)?;
            if !next.starts_with(')') {
                return Err("Expected ) after function type return".to_string());
            }
            Ok((Type::Fun(params, Box::new(ret_ty)), &next[1..]))
        }
        _ => Err(format!("Unknown type prefix: {}", first)),
    }
}

pub fn type_from_str(s: &str) -> Result<Type, String> {
    let (ty, rest) = deserialize_type(s)?;
    if !rest.is_empty() {
        return Err(format!("Unparsed type characters: {}", rest));
    }
    Ok(ty)
}

fn serialize_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(v) => format!("I:{}", v),
        Literal::Float(v) => format!("F:{}", v),
        Literal::Str(s) => format!("S:{}", hex_encode(s)),
        Literal::Bool(b) => format!("B:{}", b),
    }
}

fn deserialize_literal(s: &str) -> Result<Literal, String> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Invalid literal format".to_string());
    }
    match parts[0] {
        "I" => {
            let v = parts[1].parse::<i64>().map_err(|e| format!("Int parse error: {}", e))?;
            Ok(Literal::Int(v))
        }
        "F" => {
            let v = parts[1].parse::<f64>().map_err(|e| format!("Float parse error: {}", e))?;
            Ok(Literal::Float(v))
        }
        "S" => {
            let s = hex_decode(parts[1])?;
            Ok(Literal::Str(s))
        }
        "B" => {
            let v = parts[1].parse::<bool>().map_err(|e| format!("Bool parse error: {}", e))?;
            Ok(Literal::Bool(v))
        }
        _ => Err(format!("Unknown literal tag: {}", parts[0])),
    }
}

fn serialize_address(addr: &Address) -> String {
    match addr {
        Address::Variable(name, ty) => format!("V:{}:{}", hex_encode(name), serialize_type(ty)),
        Address::Constant(lit, ty) => format!("C:{}:{}", serialize_literal(lit), serialize_type(ty)),
        Address::Temporary(name, ty) => format!("T:{}:{}", hex_encode(name), serialize_type(ty)),
        Address::FunctionLabel(name) => format!("L:{}", hex_encode(name)),
    }
}

fn deserialize_address(s: &str) -> Result<Address, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.is_empty() {
        return Err("Empty address".to_string());
    }
    match parts[0] {
        "V" => {
            if parts.len() != 3 {
                return Err("Invalid Variable Address format".to_string());
            }
            let name = hex_decode(parts[1])?;
            let ty = type_from_str(parts[2])?;
            Ok(Address::Variable(name, ty))
        }
        "C" => {
            if parts.len() != 4 {
                return Err("Invalid Constant Address format".to_string());
            }
            let lit_str = format!("{}:{}", parts[1], parts[2]);
            let lit = deserialize_literal(&lit_str)?;
            let ty = type_from_str(parts[3])?;
            Ok(Address::Constant(lit, ty))
        }
        "T" => {
            if parts.len() != 3 {
                return Err("Invalid Temporary Address format".to_string());
            }
            let name = hex_decode(parts[1])?;
            let ty = type_from_str(parts[2])?;
            Ok(Address::Temporary(name, ty))
        }
        "L" => {
            if parts.len() != 2 {
                return Err("Invalid FunctionLabel Address format".to_string());
            }
            let name = hex_decode(parts[1])?;
            Ok(Address::FunctionLabel(name))
        }
        _ => Err(format!("Unknown address tag: {}", parts[0])),
    }
}

fn serialize_opt_address(opt: &Option<Address>) -> String {
    match opt {
        Some(addr) => format!("Some:{}", serialize_address(addr)),
        None => "None".to_string(),
    }
}

fn deserialize_opt_address(s: &str) -> Result<Option<Address>, String> {
    if s == "None" {
        Ok(None)
    } else if s.starts_with("Some:") {
        let addr_str = &s[5..];
        let addr = deserialize_address(addr_str)?;
        Ok(Some(addr))
    } else {
        Err(format!("Invalid Option<Address> string: {}", s))
    }
}

fn serialize_operator(op: &Operator) -> String {
    format!("{:?}", op)
}

fn deserialize_operator(s: &str) -> Result<Operator, String> {
    match s {
        "Add" => Ok(Operator::Add),
        "Sub" => Ok(Operator::Sub),
        "Mul" => Ok(Operator::Mul),
        "Div" => Ok(Operator::Div),
        "Neg" => Ok(Operator::Neg),
        "LT" => Ok(Operator::LT),
        "LTE" => Ok(Operator::LTE),
        "GT" => Ok(Operator::GT),
        "GTE" => Ok(Operator::GTE),
        "EQ" => Ok(Operator::EQ),
        "NE" => Ok(Operator::NE),
        "SL" => Ok(Operator::SL),
        "SR" => Ok(Operator::SR),
        _ => Err(format!("Unknown operator: {}", s)),
    }
}

fn serialize_instruction(inst: &Instruction) -> String {
    match inst {
        Instruction::Label(lbl) => {
            format!("Label|{}", hex_encode(lbl))
        }
        Instruction::CopyAssignment(dst, src) => {
            format!("CopyAssignment|{}|{}", serialize_address(dst), serialize_address(src))
        }
        Instruction::UnaryAssignment(op, dst, src) => {
            format!("UnaryAssignment|{}|{}|{}", serialize_operator(op), serialize_address(dst), serialize_address(src))
        }
        Instruction::BinaryAssignment(op, dst, src1, src2) => {
            format!("BinaryAssignment|{}|{}|{}|{}", serialize_operator(op), serialize_address(dst), serialize_address(src1), serialize_address(src2))
        }
        Instruction::JMP(lbl) => {
            format!("JMP|{}", hex_encode(lbl))
        }
        Instruction::ConditionalJMP(addr, lbl) => {
            format!("ConditionalJMP|{}|{}", serialize_address(addr), hex_encode(lbl))
        }
        Instruction::ConditionalJMPFalse(addr, lbl) => {
            format!("ConditionalJMPFalse|{}|{}", serialize_address(addr), hex_encode(lbl))
        }
        Instruction::ConditionalJMPRelational(op, src1, src2, lbl) => {
            format!("ConditionalJMPRelational|{}|{}|{}|{}", serialize_operator(op), serialize_address(src1), serialize_address(src2), hex_encode(lbl))
        }
        Instruction::Param(addr) => {
            format!("Param|{}", serialize_address(addr))
        }
        Instruction::Call(dst, name, arity) => {
            format!("Call|{}|{}|{}", serialize_opt_address(dst), hex_encode(name), arity)
        }
        Instruction::CallIndirect(dst, callee, arity) => {
            format!("CallIndirect|{}|{}|{}", serialize_opt_address(dst), serialize_address(callee), arity)
        }
        Instruction::Store(arr, idx, val) => {
            format!("Store|{}|{}|{}", serialize_address(arr), serialize_address(idx), serialize_address(val))
        }
        Instruction::Load(dst, arr, idx) => {
            format!("Load|{}|{}|{}", serialize_address(dst), serialize_address(arr), serialize_address(idx))
        }
        Instruction::Return(val) => {
            format!("Return|{}", serialize_opt_address(val))
        }
    }
}

fn deserialize_instruction(line: &str) -> Result<Instruction, String> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.is_empty() {
        return Err("Empty line in TAC program".to_string());
    }
    match parts[0] {
        "Label" => {
            if parts.len() != 2 {
                return Err("Invalid Label instruction format".to_string());
            }
            let lbl = hex_decode(parts[1])?;
            Ok(Instruction::Label(lbl))
        }
        "CopyAssignment" => {
            if parts.len() != 3 {
                return Err("Invalid CopyAssignment instruction format".to_string());
            }
            let dst = deserialize_address(parts[1])?;
            let src = deserialize_address(parts[2])?;
            Ok(Instruction::CopyAssignment(dst, src))
        }
        "UnaryAssignment" => {
            if parts.len() != 4 {
                return Err("Invalid UnaryAssignment instruction format".to_string());
            }
            let op = deserialize_operator(parts[1])?;
            let dst = deserialize_address(parts[2])?;
            let src = deserialize_address(parts[3])?;
            Ok(Instruction::UnaryAssignment(op, dst, src))
        }
        "BinaryAssignment" => {
            if parts.len() != 5 {
                return Err("Invalid BinaryAssignment instruction format".to_string());
            }
            let op = deserialize_operator(parts[1])?;
            let dst = deserialize_address(parts[2])?;
            let src1 = deserialize_address(parts[3])?;
            let src2 = deserialize_address(parts[4])?;
            Ok(Instruction::BinaryAssignment(op, dst, src1, src2))
        }
        "JMP" => {
            if parts.len() != 2 {
                return Err("Invalid JMP instruction format".to_string());
            }
            let lbl = hex_decode(parts[1])?;
            Ok(Instruction::JMP(lbl))
        }
        "ConditionalJMP" => {
            if parts.len() != 3 {
                return Err("Invalid ConditionalJMP instruction format".to_string());
            }
            let addr = deserialize_address(parts[1])?;
            let lbl = hex_decode(parts[2])?;
            Ok(Instruction::ConditionalJMP(addr, lbl))
        }
        "ConditionalJMPFalse" => {
            if parts.len() != 3 {
                return Err("Invalid ConditionalJMPFalse instruction format".to_string());
            }
            let addr = deserialize_address(parts[1])?;
            let lbl = hex_decode(parts[2])?;
            Ok(Instruction::ConditionalJMPFalse(addr, lbl))
        }
        "ConditionalJMPRelational" => {
            if parts.len() != 5 {
                return Err("Invalid ConditionalJMPRelational instruction format".to_string());
            }
            let op = deserialize_operator(parts[1])?;
            let src1 = deserialize_address(parts[2])?;
            let src2 = deserialize_address(parts[3])?;
            let lbl = hex_decode(parts[4])?;
            Ok(Instruction::ConditionalJMPRelational(op, src1, src2, lbl))
        }
        "Param" => {
            if parts.len() != 2 {
                return Err("Invalid Param instruction format".to_string());
            }
            let addr = deserialize_address(parts[1])?;
            Ok(Instruction::Param(addr))
        }
        "Call" => {
            if parts.len() != 4 {
                return Err("Invalid Call instruction format".to_string());
            }
            let dst = deserialize_opt_address(parts[1])?;
            let name = hex_decode(parts[2])?;
            let arity = parts[3].parse::<usize>().map_err(|e| format!("Call arity parse error: {}", e))?;
            Ok(Instruction::Call(dst, name, arity))
        }
        "CallIndirect" => {
            if parts.len() != 4 {
                return Err("Invalid CallIndirect instruction format".to_string());
            }
            let dst = deserialize_opt_address(parts[1])?;
            let callee = deserialize_address(parts[2])?;
            let arity = parts[3].parse::<usize>().map_err(|e| format!("CallIndirect arity parse error: {}", e))?;
            Ok(Instruction::CallIndirect(dst, callee, arity))
        }
        "Store" => {
            if parts.len() != 4 {
                return Err("Invalid Store instruction format".to_string());
            }
            let arr = deserialize_address(parts[1])?;
            let idx = deserialize_address(parts[2])?;
            let val = deserialize_address(parts[3])?;
            Ok(Instruction::Store(arr, idx, val))
        }
        "Load" => {
            if parts.len() != 4 {
                return Err("Invalid Load instruction format".to_string());
            }
            let dst = deserialize_address(parts[1])?;
            let arr = deserialize_address(parts[2])?;
            let idx = deserialize_address(parts[3])?;
            Ok(Instruction::Load(dst, arr, idx))
        }
        "Return" => {
            if parts.len() != 2 {
                return Err("Invalid Return instruction format".to_string());
            }
            let val = deserialize_opt_address(parts[1])?;
            Ok(Instruction::Return(val))
        }
        _ => Err(format!("Unknown instruction tag: {}", parts[0])),
    }
}

pub fn serialize_tac(program: &TACProgram) -> String {
    let mut lines = Vec::new();
    for inst in program {
        lines.push(serialize_instruction(inst));
    }
    lines.join("\n")
}

pub fn deserialize_tac(s: &str) -> Result<TACProgram, String> {
    let mut program = Vec::new();
    for (line_idx, line) in s.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let inst = deserialize_instruction(trimmed)
            .map_err(|e| format!("Error parsing line {}: {}", line_idx + 1, e))?;
        program.push(inst);
    }
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::ast::{Literal, Type};

    #[test]
    fn test_display_operator() {
        assert_eq!(format!("{}", Operator::Add), "+");
        assert_eq!(format!("{}", Operator::NE), "!=");
    }

    #[test]
    fn test_display_address() {
        let var = Address::Variable("x".to_string(), Type::Int);
        assert_eq!(format!("{}", var), "x");

        let con = Address::Constant(Literal::Int(42), Type::Int);
        assert_eq!(format!("{}", con), "42");

        let lambda = Address::FunctionLabel("lambda_0".to_string());
        assert_eq!(format!("{}", lambda), "lambda_0");
    }

    #[test]
    fn test_display_instruction() {
        let copy = Instruction::CopyAssignment(
            Address::Variable("double".to_string(), Type::Fun(vec![Type::Int], Box::new(Type::Int))),
            Address::FunctionLabel("lambda_0".to_string())
        );
        assert_eq!(format!("{}", copy), "double = lambda_0");

        let call_ind = Instruction::CallIndirect(
            None,
            Address::Variable("double".to_string(), Type::Fun(vec![Type::Int], Box::new(Type::Int))),
            1
        );
        assert_eq!(format!("{}", call_ind), "call_indirect double, 1");

        let label = Instruction::Label("lambda_0".to_string());
        assert_eq!(format!("{}", label), "Label lambda_0");
    }

    #[test]
    fn test_serialization_round_trip() {
        let prog = vec![
            Instruction::Label("lambda_0".to_string()),
            Instruction::BinaryAssignment(
                Operator::Mul,
                Address::Temporary("temp1".to_string(), Type::Int),
                Address::Variable("x".to_string(), Type::Int),
                Address::Constant(Literal::Int(2), Type::Int)
            ),
            Instruction::Return(Some(Address::Temporary("temp1".to_string(), Type::Int))),
            Instruction::Label("Label1:".to_string()),
            Instruction::CopyAssignment(
                Address::Variable("double".to_string(), Type::Fun(vec![Type::Int], Box::new(Type::Int))),
                Address::FunctionLabel("lambda_0".to_string())
            ),
            Instruction::Param(Address::Constant(Literal::Int(21), Type::Int)),
            Instruction::CallIndirect(
                Some(Address::Temporary("temp2".to_string(), Type::Int)),
                Address::Variable("double".to_string(), Type::Fun(vec![Type::Int], Box::new(Type::Int))),
                1
            )
        ];

        let serialized = serialize_tac(&prog);
        let deserialized = deserialize_tac(&serialized).expect("Deserialization failed");
        assert_eq!(prog, deserialized);
    }
}
