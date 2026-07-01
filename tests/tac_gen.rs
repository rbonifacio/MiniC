//! Integration tests for the MiniC TAC generator.
//!
//! Two test styles:
//! 1. Build a typed AST manually (helpers) and call `translate_statement`.
//! 2. Parse + type-check a `.minic` file and call `translate_program`.

use mini_c::codegen::tac_code_gen::{translate_program, Environment, translate_statement};
use mini_c::ir::ast::{CheckedExpr, CheckedStmt, Expr, ExprD, Literal, Statement, StatementD, Type};
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::parser::program;
use mini_c::semantic::type_check;

// --- Helpers: build typed AST nodes without going through the parser ---

/// Typed expression: integer variable (e.g. `x` with type `int`).
fn int_var(name: &str) -> CheckedExpr {
    ExprD {
        exp: Expr::Ident(name.to_string()),
        ty: Type::Int,
    }
}

/// Typed expression: sum of two integers (`left + right`).
fn add(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD {
        exp: Expr::Add(Box::new(left), Box::new(right)),
        ty: Type::Int,
    }
}

/// Typed expression: comparison `left < right` (result type `bool`).
fn lt(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD {
        exp: Expr::Lt(Box::new(left), Box::new(right)),
        ty: Type::Bool,
    }
}

/// Typed expression: integer literal (e.g. `42`).
fn int_lit(n: i64) -> CheckedExpr {
    ExprD {
        exp: Expr::Literal(Literal::Int(n)),
        ty: Type::Int,
    }
}

/// Typed statement: declaration `int name = init`.
fn decl(name: &str, init: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Decl {
            name: name.to_string(),
            ty: Type::Int,
            init: Box::new(init),
        },
        ty: Type::Unit, // declaration produces no value
    }
}

/// Typed statement: loop `for (init; cond; update) body`.
/// Each header clause may be `None` (omitted).
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

/// Typed statement: assignment `name = value`.
fn assign(name: &str, value: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Assign {
            target: Box::new(ExprD {
                exp: Expr::Ident(name.to_string()),
                ty: value.ty.clone(), // target type = expression type
            }),
            value: Box::new(value),
        },
        ty: Type::Unit,
    }
}

// --- Test: if-else with relational condition ---
//
// MiniC:
//   if (x < y) { z = x + y; } else { z = x; }
//
// Expected TAC (separate labels for then / else / end):
//   if x < y goto Label1:   ← then
//   goto Label2:             ← else
//   Label1:
//   temp1 = x + y
//   z = temp1
//   goto Label3:             ← end (skip else)
//   Label2:
//   z = x
//   Label3:
#[test]
fn test_if_else_with_relational_condition() {
    // Build the typed AST `If` node.
    let stmt = StatementD {
        stmt: Statement::If {
            cond: Box::new(lt(int_var("x"), int_var("y"))),
            then_branch: Box::new(assign("z", add(int_var("x"), int_var("y")))),
            else_branch: Some(Box::new(assign("z", int_var("x")))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    // Expected addresses in instructions (for exact comparison).
    let x = Address::Variable("x".to_string(), Type::Int);
    let y = Address::Variable("y".to_string(), Type::Int);
    let z = Address::Variable("z".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(
        instructions,
        vec![
            Instruction::ConditionalJMPRelational(
                Operator::LT,
                x.clone(),
                y.clone(),
                "Label1:".to_string(),
            ),
            Instruction::JMP("Label2:".to_string()),
            Instruction::Label("Label1:".to_string()),
            Instruction::BinaryAssignment(Operator::Add, temp.clone(), x.clone(), y.clone()),
            Instruction::CopyAssignment(z.clone(), temp),
            Instruction::JMP("Label3:".to_string()),
            Instruction::Label("Label2:".to_string()),
            Instruction::CopyAssignment(z, x),
            Instruction::Label("Label3:".to_string()),
        ]
    );
}

// --- Test: canonical for loop ---
//
// MiniC:
//   for (int i = 0; i < 10; i = i + 1) { sum = sum + i; }
//
// Expected TAC:
//   i = 0
//   Label1:                  ← condition test
//   if i < 10 goto Label3:   ← true → body
//   goto Label2:              ← false → exit
//   Label3:                   ← body entry
//   temp1 = sum + i
//   sum = temp1
//   temp2 = i + 1
//   i = temp2
//   goto Label1:
//   Label2:
#[test]
fn test_for_canonical() {
    let stmt = for_stmt(
        Some(decl("i", int_lit(0))),                        // init
        Some(lt(int_var("i"), int_lit(10))),                // cond
        Some(assign("i", add(int_var("i"), int_lit(1)))),    // update
        assign("sum", add(int_var("sum"), int_var("i"))),  // body
    );

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let i = Address::Variable("i".to_string(), Type::Int);
    let sum = Address::Variable("sum".to_string(), Type::Int);
    let zero = Address::Constant(Literal::Int(0), Type::Int);
    let ten = Address::Constant(Literal::Int(10), Type::Int);
    let one = Address::Constant(Literal::Int(1), Type::Int);
    let temp1 = Address::Temporary("temp1".to_string(), Type::Int);
    let temp2 = Address::Temporary("temp2".to_string(), Type::Int);

    assert_eq!(
        instructions,
        vec![
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
        ]
    );
}

// --- Test: infinite for loop (omitted condition) ---
//
// MiniC:
//   for (;;) { sum = sum + 1; }
//
// Expected TAC: loop with no exit test; Label2 only marks structural end.
#[test]
fn test_for_infinite_loop() {
    let stmt = for_stmt(
        None, // no init
        None, // no cond → infinite loop
        None, // no update
        assign("sum", add(int_var("sum"), int_lit(1))),
    );

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let sum = Address::Variable("sum".to_string(), Type::Int);
    let one = Address::Constant(Literal::Int(1), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(
        instructions,
        vec![
            Instruction::Label("Label1:".to_string()),
            Instruction::BinaryAssignment(Operator::Add, temp.clone(), sum.clone(), one),
            Instruction::CopyAssignment(sum, temp),
            Instruction::JMP("Label1:".to_string()),
            Instruction::Label("Label2:".to_string()),
        ]
    );
}

// --- End-to-end test: .minic file → parse → type-check → TAC ---
//
// Uses `tests/fixtures/tac_simple.minic` and compares textual output (`Display`).
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
