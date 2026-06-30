//! Integration tests for the MiniC TAC code generator.

use mini_c::ir::ast::{CheckedExpr, CheckedStmt, ExprD, Expr, Literal, Statement, StatementD, Type};
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::codegen::tac_code_gen::{Environment, translate_statement};

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
        Instruction::ConditionalJMPRelational(Operator::LT, x.clone(), y.clone(), "Label1:".to_string()),
        Instruction::JMP("Label2:".to_string()),
        Instruction::Label("Label1:".to_string()),
        Instruction::BinaryAssignment(Operator::Add, temp.clone(), x.clone(), y.clone()),
        Instruction::CopyAssignment(z.clone(), temp),
        Instruction::JMP("Label3:".to_string()),
        Instruction::Label("Label2:".to_string()),
        Instruction::CopyAssignment(z, x),
        Instruction::Label("Label3:".to_string()),
    ]);
}

#[test]
fn test_switch_int_tac() {
    let stmt = StatementD {
        stmt: Statement::Switch {
            target: Box::new(int_var("x")),
            cases: vec![
                (Literal::Int(1), vec![assign("z", ExprD { exp: Expr::Literal(Literal::Int(10)), ty: Type::Int })]),
                (Literal::Int(2), vec![assign("z", ExprD { exp: Expr::Literal(Literal::Int(20)), ty: Type::Int })]),
            ],
            default: vec![
                assign("z", ExprD { exp: Expr::Literal(Literal::Int(30)), ty: Type::Int })
            ],
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let x = Address::Variable("x".to_string(), Type::Int);
    let z = Address::Variable("z".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        // comparisons
        Instruction::ConditionalJMPRelational(Operator::EQ, x.clone(), Address::Constant(Literal::Int(1), Type::Int), "Label3:".to_string()),
        Instruction::ConditionalJMPRelational(Operator::EQ, x.clone(), Address::Constant(Literal::Int(2), Type::Int), "Label4:".to_string()),
        // fallback to default
        Instruction::JMP("Label1:".to_string()),
        // case 1
        Instruction::Label("Label3:".to_string()),
        Instruction::CopyAssignment(z.clone(), Address::Constant(Literal::Int(10), Type::Int)),
        Instruction::JMP("Label2:".to_string()),
        // case 2
        Instruction::Label("Label4:".to_string()),
        Instruction::CopyAssignment(z.clone(), Address::Constant(Literal::Int(20), Type::Int)),
        Instruction::JMP("Label2:".to_string()),
        // default
        Instruction::Label("Label1:".to_string()),
        Instruction::CopyAssignment(z.clone(), Address::Constant(Literal::Int(30), Type::Int)),
        // end
        Instruction::Label("Label2:".to_string()),
    ]);
}

#[test]
fn test_switch_bool_tac() {
    let stmt = StatementD {
        stmt: Statement::Switch {
            target: Box::new(ExprD { exp: Expr::Ident("b".to_string()), ty: Type::Bool }),
            cases: vec![
                (Literal::Bool(true), vec![assign("z", ExprD { exp: Expr::Literal(Literal::Int(1)), ty: Type::Int })]),
            ],
            default: vec![
                assign("z", ExprD { exp: Expr::Literal(Literal::Int(0)), ty: Type::Int })
            ],
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let b = Address::Variable("b".to_string(), Type::Bool);
    let z = Address::Variable("z".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        // comparisons
        Instruction::ConditionalJMPRelational(Operator::EQ, b.clone(), Address::Constant(Literal::Bool(true), Type::Bool), "Label3:".to_string()),
        // fallback to default
        Instruction::JMP("Label1:".to_string()),
        // case true
        Instruction::Label("Label3:".to_string()),
        Instruction::CopyAssignment(z.clone(), Address::Constant(Literal::Int(1), Type::Int)),
        Instruction::JMP("Label2:".to_string()),
        // default
        Instruction::Label("Label1:".to_string()),
        Instruction::CopyAssignment(z.clone(), Address::Constant(Literal::Int(0), Type::Int)),
        // end
        Instruction::Label("Label2:".to_string()),
    ]);
}

#[test]
fn test_switch_complex_target_tac() {
    let stmt = StatementD {
        stmt: Statement::Switch {
            target: Box::new(add(int_var("x"), int_var("y"))),
            cases: vec![
                (Literal::Int(5), vec![assign("z", ExprD { exp: Expr::Literal(Literal::Int(50)), ty: Type::Int })]),
            ],
            default: vec![
                assign("z", ExprD { exp: Expr::Literal(Literal::Int(999)), ty: Type::Int })
            ],
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let x = Address::Variable("x".to_string(), Type::Int);
    let y = Address::Variable("y".to_string(), Type::Int);
    let z = Address::Variable("z".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        // evaluate target expression
        Instruction::BinaryAssignment(Operator::Add, temp.clone(), x, y),
        // comparisons
        Instruction::ConditionalJMPRelational(Operator::EQ, temp.clone(), Address::Constant(Literal::Int(5), Type::Int), "Label3:".to_string()),
        // fallback to default
        Instruction::JMP("Label1:".to_string()),
        // case 5
        Instruction::Label("Label3:".to_string()),
        Instruction::CopyAssignment(z.clone(), Address::Constant(Literal::Int(50), Type::Int)),
        Instruction::JMP("Label2:".to_string()),
        // default
        Instruction::Label("Label1:".to_string()),
        Instruction::CopyAssignment(z.clone(), Address::Constant(Literal::Int(999), Type::Int)),
        // end
        Instruction::Label("Label2:".to_string()),
    ]);
}
