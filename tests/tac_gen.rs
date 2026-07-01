//! Integration tests for the MiniC TAC code generator.

use mini_c::ir::ast::{CheckedExpr, CheckedStmt, ExprD, Expr, Literal, Statement, StatementD, Type};
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::codegen::tac_code_gen::{Environment, translate_statement};
use mini_c::codegen::linker::Linker;

// --- Helpers to build type-annotated AST nodes ---

fn int_var(name: &str) -> CheckedExpr {
    ExprD { exp: Expr::Ident(name.to_string()), ty: Type::Int }
}

fn str_var(name: &str) -> CheckedExpr {
    ExprD { exp: Expr::Ident(name.to_string()), ty: Type::Str }
}

fn str_lit(s: &str) -> CheckedExpr {
    ExprD { exp: Expr::Literal(Literal::Str(s.to_string())), ty: Type::Str }
}

fn int_lit(n: i64) -> CheckedExpr {
    ExprD { exp: Expr::Literal(Literal::Int(n)), ty: Type::Int }
}

fn add(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD { exp: Expr::Add(Box::new(left), Box::new(right)), ty: Type::Int }
}

fn lt(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD { exp: Expr::Lt(Box::new(left), Box::new(right)), ty: Type::Bool }
}

fn concat(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD { exp: Expr::Concat(Box::new(left), Box::new(right)), ty: Type::Str }
}

fn len_of(arg: CheckedExpr) -> CheckedExpr {
    ExprD { exp: Expr::Len(Box::new(arg)), ty: Type::Int }
}

fn contains_of(container: CheckedExpr, item: CheckedExpr) -> CheckedExpr {
    ExprD { exp: Expr::Contains(Box::new(container), Box::new(item)), ty: Type::Bool }
}

fn call(name: &str, args: Vec<CheckedExpr>, ret: Type) -> CheckedExpr {
    ExprD { exp: Expr::Call { name: name.to_string(), args }, ty: ret }
}

fn assign(name: &str, value: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Assign {
            target: Box::new(ExprD { exp: Expr::Ident(name.to_string()), ty: value.ty.clone() }),
            value: Box::new(value),
        },
        ty: Type::Unit,
    }
}

// Fixture: if (x < y) { z = x + y; } else { z = x; }
//
// Expected TAC:
//   if x >= y goto Label1:    <- negated relational jumps to else
//   temp1 = x + y
//   z = temp1
//   goto Label2:
//   Label1:
//   z = x
//   Label2:
#[test]
fn test_if_else_with_relational_condition() {
    let stmt = StatementD {
        stmt: Statement::If {
            cond:        Box::new(lt(int_var("x"), int_var("y"))),
            then_branch: Box::new(assign("z", add(int_var("x"), int_var("y")))),
            else_branch: Some(Box::new(assign("z", int_var("x")))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let x    = Address::Variable("x".to_string(), Type::Int);
    let y    = Address::Variable("y".to_string(), Type::Int);
    let z    = Address::Variable("z".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::ConditionalJMPRelational(Operator::GTE, x.clone(), y.clone(), "Label1:".to_string()),
        Instruction::BinaryAssignment(Operator::Add, temp.clone(), x.clone(), y.clone()),
        Instruction::CopyAssignment(z.clone(), temp),
        Instruction::JMP("Label2:".to_string()),
        Instruction::Label("Label1:".to_string()),
        Instruction::CopyAssignment(z, x),
        Instruction::Label("Label2:".to_string()),
    ]);
}

// Fixture: s = a ++ b;
//
// Expected TAC:
//   temp1 = a ++ b    # BinaryAssignment(Concat, temp1, a, b)
//   s     = temp1     # CopyAssignment(s, temp1)
#[test]
fn test_concat() {
    let stmt = assign("s", concat(str_var("a"), str_var("b")));

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let a    = Address::Variable("a".to_string(), Type::Str);
    let b    = Address::Variable("b".to_string(), Type::Str);
    let s    = Address::Variable("s".to_string(), Type::Str);
    let temp = Address::Temporary("temp1".to_string(), Type::Str);

    assert_eq!(instructions, vec![
        Instruction::BinaryAssignment(Operator::Concat, temp.clone(), a, b),
        Instruction::CopyAssignment(s, temp),
    ]);
}

// Fixture: n = len(s);
//
// Expected TAC:
//   temp1 = len(s)    # UnaryAssignment(Len, temp1, s)
//   n     = temp1     # CopyAssignment(n, temp1)
#[test]
fn test_len() {
    let stmt = assign("n", len_of(str_var("s")));

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let s    = Address::Variable("s".to_string(), Type::Str);
    let n    = Address::Variable("n".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::UnaryAssignment(Operator::Len, temp.clone(), s),
        Instruction::CopyAssignment(n, temp),
    ]);
}

// Fixture: b = contains(s, sub);
//
// Expected TAC:
//   temp1 = contains(s, sub)    # BinaryAssignment(Contains, temp1, s, sub)
//   b     = temp1               # CopyAssignment(b, temp1)
#[test]
fn test_contains() {
    let stmt = assign("b", contains_of(str_var("s"), str_var("sub")));

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let s    = Address::Variable("s".to_string(), Type::Str);
    let sub  = Address::Variable("sub".to_string(), Type::Str);
    let b    = Address::Variable("b".to_string(), Type::Bool);
    let temp = Address::Temporary("temp1".to_string(), Type::Bool);

    assert_eq!(instructions, vec![
        Instruction::BinaryAssignment(Operator::Contains, temp.clone(), s, sub),
        Instruction::CopyAssignment(b, temp),
    ]);
}

// Fixture: result = toUpper(s);
//
// Expected TAC:
//   param s                      # Param(s)
//   temp1 = call toUpper, 1      # Call(Some(temp1), "toUpper", 1)
//   result = temp1               # CopyAssignment(result, temp1)
#[test]
fn test_call_to_upper() {
    let stmt = assign("result", call("toUpper", vec![str_var("s")], Type::Str));

    let linker = Linker::new();
    let mut env = Environment::with_linker(&linker);
    let instructions = translate_statement(stmt, &mut env);

    let s      = Address::Variable("s".to_string(), Type::Str);
    let result = Address::Variable("result".to_string(), Type::Str);
    let temp   = Address::Temporary("temp1".to_string(), Type::Str);

    assert_eq!(instructions, vec![
        Instruction::Param(s),
        Instruction::ExternCall(Some(temp.clone()), "toUpper".to_string(), 1),
        Instruction::CopyAssignment(result, temp),
    ]);
}

// Fixture: result = toLower(s);
//
// Expected TAC:
//   param s                      # Param(s)
//   temp1 = call toLower, 1      # Call(Some(temp1), "toLower", 1)
//   result = temp1               # CopyAssignment(result, temp1)
#[test]
fn test_call_to_lower() {
    let stmt = assign("result", call("toLower", vec![str_var("s")], Type::Str));

    let linker = Linker::new();
    let mut env = Environment::with_linker(&linker);
    let instructions = translate_statement(stmt, &mut env);

    let s      = Address::Variable("s".to_string(), Type::Str);
    let result = Address::Variable("result".to_string(), Type::Str);
    let temp   = Address::Temporary("temp1".to_string(), Type::Str);

    assert_eq!(instructions, vec![
        Instruction::Param(s),
        Instruction::ExternCall(Some(temp.clone()), "toLower".to_string(), 1),
        Instruction::CopyAssignment(result, temp),
    ]);
}

// Fixture: n = strToInt(s);
//
// Expected TAC:
//   param s                      # Param(s)
//   temp1 = call strToInt, 1     # Call(Some(temp1), "strToInt", 1)
//   n     = temp1                # CopyAssignment(n, temp1)
#[test]
fn test_call_str_to_int() {
    let stmt = assign("n", call("strToInt", vec![str_var("s")], Type::Int));

    let linker = Linker::new();
    let mut env = Environment::with_linker(&linker);
    let instructions = translate_statement(stmt, &mut env);

    let s    = Address::Variable("s".to_string(), Type::Str);
    let n    = Address::Variable("n".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::Param(s),
        Instruction::ExternCall(Some(temp.clone()), "strToInt".to_string(), 1),
        Instruction::CopyAssignment(n, temp),
    ]);
}

// Fixture: x = strToFloat(s);
//
// Expected TAC:
//   param s                       # Param(s)
//   temp1 = call strToFloat, 1    # Call(Some(temp1), "strToFloat", 1)
//   x     = temp1                 # CopyAssignment(x, temp1)
#[test]
fn test_call_str_to_float() {
    let stmt = assign("x", call("strToFloat", vec![str_var("s")], Type::Float));

    let linker = Linker::new();
    let mut env = Environment::with_linker(&linker);
    let instructions = translate_statement(stmt, &mut env);

    let s    = Address::Variable("s".to_string(), Type::Str);
    let x    = Address::Variable("x".to_string(), Type::Float);
    let temp = Address::Temporary("temp1".to_string(), Type::Float);

    assert_eq!(instructions, vec![
        Instruction::Param(s),
        Instruction::ExternCall(Some(temp.clone()), "strToFloat".to_string(), 1),
        Instruction::CopyAssignment(x, temp),
    ]);
}

// Fixture: result = substr(s, 0, 3);
//
// Expected TAC:
//   param s                        # Param(s)
//   param 0                        # Param(0)
//   param 3                        # Param(3)
//   temp1 = call substr, 3         # Call(Some(temp1), "substr", 3)
//   result = temp1                 # CopyAssignment(result, temp1)
#[test]
fn test_call_substr() {
    let stmt = assign("result", call("substr", vec![str_var("s"), int_lit(0), int_lit(3)], Type::Str));

    let linker = Linker::new();
    let mut env = Environment::with_linker(&linker);
    let instructions = translate_statement(stmt, &mut env);

    let s      = Address::Variable("s".to_string(), Type::Str);
    let zero   = Address::Constant(Literal::Int(0), Type::Int);
    let three  = Address::Constant(Literal::Int(3), Type::Int);
    let result = Address::Variable("result".to_string(), Type::Str);
    let temp   = Address::Temporary("temp1".to_string(), Type::Str);

    assert_eq!(instructions, vec![
        Instruction::Param(s),
        Instruction::Param(zero),
        Instruction::Param(three),
        Instruction::ExternCall(Some(temp.clone()), "substr".to_string(), 3),
        Instruction::CopyAssignment(result, temp),
    ]);
}


// --- Helpers to build type-annotated AST nodes ---

fn int_var(name: &str) -> CheckedExpr {
    ExprD { exp: Expr::Ident(name.to_string()), ty: Type::Int }
}

fn add(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD { exp: Expr::Add(Box::new(left), Box::new(right)), ty: Type::Int }
}

fn lt(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD { exp: Expr::Lt(Box::new(left), Box::new(right)), ty: Type::Bool }
}

fn assign(name: &str, value: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Assign {
            target: Box::new(ExprD { exp: Expr::Ident(name.to_string()), ty: value.ty.clone() }),
            value: Box::new(value),
        },
        ty: Type::Unit,
    }
}

// Fixture: if (x < y) { z = x + y; } else { z = x; }
//
// Expected TAC:
//   if x >= y goto Label1:    <- negated relational jumps to else
//   temp1 = x + y
//   z = temp1
//   goto Label2:
//   Label1:
//   z = x
//   Label2:
#[test]
fn test_if_else_with_relational_condition() {
    let stmt = StatementD {
        stmt: Statement::If {
            cond:        Box::new(lt(int_var("x"), int_var("y"))),
            then_branch: Box::new(assign("z", add(int_var("x"), int_var("y")))),
            else_branch: Some(Box::new(assign("z", int_var("x")))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let x    = Address::Variable("x".to_string(), Type::Int);
    let y    = Address::Variable("y".to_string(), Type::Int);
    let z    = Address::Variable("z".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::ConditionalJMPRelational(Operator::GTE, x.clone(), y.clone(), "Label1:".to_string()),
        Instruction::BinaryAssignment(Operator::Add, temp.clone(), x.clone(), y.clone()),
        Instruction::CopyAssignment(z.clone(), temp),
        Instruction::JMP("Label2:".to_string()),
        Instruction::Label("Label1:".to_string()),
        Instruction::CopyAssignment(z, x),
        Instruction::Label("Label2:".to_string()),
    ]);
}
