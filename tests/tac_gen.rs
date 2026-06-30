//! Integration tests for the MiniC TAC code generator.

use mini_c::codegen::tac_code_gen::{translate_program, Environment, translate_statement};
use mini_c::ir::ast::{CheckedExpr, CheckedStmt, Expr, ExprD, Literal, Statement, StatementD, Type};
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::parser::program;
use mini_c::semantic::type_check;

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

fn int_lit(n: i64) -> CheckedExpr {
    ExprD { exp: Expr::Literal(Literal::Int(n)), ty: Type::Int }
}

fn decl(name: &str, init: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Decl {
            name: name.to_string(),
            ty: Type::Int,
            init: Box::new(init),
        },
        ty: Type::Unit,
    }
}

fn for_stmt(
    init: Option<CheckedStmt>,
    cond: Option<CheckedExpr>,
    update: Option<CheckedStmt>,
    body: CheckedStmt,
) -> CheckedStmt {
    StatementD {
        stmt: Statement::For {
            init: init.map(Box::new),
            cond: cond.map(Box::new),
            update: update.map(Box::new),
            body: Box::new(body),
        },
        ty: Type::Unit,
    }
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

// Fixture: for (int i = 0; i < 10; i = i + 1) { sum = sum + i; }
//
// Expected TAC:
//   i := 0
//   Label1:
//   if i < 10 goto Label3:
//   goto Label2:
//   Label3:
//   temp1 = sum + i
//   sum = temp1
//   temp2 = i + 1
//   i = temp2
//   goto Label1:
//   Label2:
#[test]
fn test_for_canonical() {
    let stmt = for_stmt(
        Some(decl("i", int_lit(0))),
        Some(lt(int_var("i"), int_lit(10))),
        Some(assign("i", add(int_var("i"), int_lit(1)))),
        assign("sum", add(int_var("sum"), int_var("i"))),
    );

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let i    = Address::Variable("i".to_string(), Type::Int);
    let sum  = Address::Variable("sum".to_string(), Type::Int);
    let zero = Address::Constant(Literal::Int(0), Type::Int);
    let ten  = Address::Constant(Literal::Int(10), Type::Int);
    let one  = Address::Constant(Literal::Int(1), Type::Int);
    let temp1 = Address::Temporary("temp1".to_string(), Type::Int);
    let temp2 = Address::Temporary("temp2".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::CopyAssignment(i.clone(), zero),
        Instruction::Label("Label1:".to_string()),
        Instruction::ConditionalJMPRelational(Operator::LT, i.clone(), ten, "Label3:".to_string()),
        Instruction::JMP("Label2:".to_string()),
        Instruction::Label("Label3:".to_string()),
        Instruction::BinaryAssignment(Operator::Add, temp1.clone(), sum.clone(), i.clone()),
        Instruction::CopyAssignment(sum.clone(), temp1),
        Instruction::BinaryAssignment(Operator::Add, temp2.clone(), i.clone(), one),
        Instruction::CopyAssignment(i, temp2),
        Instruction::JMP("Label1:".to_string()),
        Instruction::Label("Label2:".to_string()),
    ]);
}

// Fixture: for (;;) { sum = sum + 1; }
//
// Expected TAC:
//   Label1:
//   temp1 = sum + 1
//   sum = temp1
//   goto Label1:
//   Label2:
#[test]
fn test_for_infinite_loop() {
    let stmt = for_stmt(
        None,
        None,
        None,
        assign("sum", add(int_var("sum"), int_lit(1))),
    );

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let sum  = Address::Variable("sum".to_string(), Type::Int);
    let one  = Address::Constant(Literal::Int(1), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::Label("Label1:".to_string()),
        Instruction::BinaryAssignment(Operator::Add, temp.clone(), sum.clone(), one),
        Instruction::CopyAssignment(sum, temp),
        Instruction::JMP("Label1:".to_string()),
        Instruction::Label("Label2:".to_string()),
    ]);
}

#[test]
fn test_translate_program_from_source() {
    let src = include_str!("fixtures/tac_simple.minic");
    let (_, unchecked) = program(src).expect("parse");
    let checked = type_check(&unchecked).expect("type-check");
    let tac = translate_program(&checked);

    assert_eq!(
        tac.iter().map(|i| i.to_string()).collect::<Vec<_>>(),
        vec![
            "main".to_string(),
            "x = 1".to_string(),
            "temp1 = x + 2".to_string(),
            "x = temp1".to_string(),
        ]
    );
}
