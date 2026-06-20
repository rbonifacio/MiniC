//! Integration tests for the MiniC TAC code generator.

use mini_c::codegen::tac_code_gen::{translate_program, translate_statement, Environment};
use mini_c::ir::ast::{
    AgtTypeSpecifier, CheckedExpr, CheckedStmt, Expr, ExprD, Literal, Statement, StatementD, Type,
};
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::parser::program;
use mini_c::semantic::type_check;
use nom::combinator::all_consuming;
use std::path::Path;

// --- Helpers to build type-annotated AST nodes ---

fn int_var(name: &str) -> CheckedExpr {
    ExprD {
        exp: Expr::Ident(name.to_string()),
        ty: Type::Int,
    }
}

fn add(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD {
        exp: Expr::Add(Box::new(left), Box::new(right)),
        ty: Type::Int,
    }
}

fn lt(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD {
        exp: Expr::Lt(Box::new(left), Box::new(right)),
        ty: Type::Bool,
    }
}

fn assign(name: &str, value: CheckedExpr) -> CheckedStmt {
    StatementD {
        stmt: Statement::Assign {
            target: Box::new(ExprD {
                exp: Expr::Ident(name.to_string()),
                ty: value.ty.clone(),
            }),
            value: Box::new(value),
        },
        ty: Type::Unit,
    }
}

fn fixtures_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
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
            cond: Box::new(lt(int_var("x"), int_var("y"))),
            then_branch: Box::new(assign("z", add(int_var("x"), int_var("y")))),
            else_branch: Some(Box::new(assign("z", int_var("x")))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let x = Address::Variable("x".to_string(), Type::Int);
    let y = Address::Variable("y".to_string(), Type::Int);
    let z = Address::Variable("z".to_string(), Type::Int);
    let temp = Address::Temporary("temp1".to_string(), Type::Int);

    assert_eq!(
        instructions,
        vec![
            Instruction::ConditionalJMPRelational(
                Operator::GTE,
                x.clone(),
                y.clone(),
                "Label1:".to_string()
            ),
            Instruction::BinaryAssignment(Operator::Add, temp.clone(), x.clone(), y.clone()),
            Instruction::CopyAssignment(z.clone(), temp),
            Instruction::JMP("Label2:".to_string()),
            Instruction::Label("Label1:".to_string()),
            Instruction::CopyAssignment(z, x),
            Instruction::Label("Label2:".to_string()),
        ]
    );
}

#[test]
fn test_aggregate_types_fixture_generates_tac() {
    let source = std::fs::read_to_string(fixtures_dir().join("aggregate_types.minic"))
        .expect("aggregate fixture should exist");
    let (_, unchecked) = all_consuming(program)(source.trim())
        .map_err(|e| e.map_input(String::from))
        .expect("aggregate fixture should parse");
    let checked = type_check(&unchecked).expect("aggregate fixture should type-check");

    let mut env = Environment::new();
    let instructions = translate_program(checked, &mut env);

    let point_ty = Type::Aggregate {
        specifier: AgtTypeSpecifier::Struct,
        identifier: "Point".to_string(),
    };
    let kind_ty = Type::Aggregate {
        specifier: AgtTypeSpecifier::Enum,
        identifier: "Kind".to_string(),
    };

    let p = Address::Variable("p".to_string(), point_ty);
    let p_valid = Address::Variable("p.valid".to_string(), Type::Bool);
    let k = Address::Variable("k".to_string(), kind_ty);
    let v = Address::Variable("v".to_string(), Type::Int);

    assert_eq!(
        instructions,
        vec![
            Instruction::Label("main".to_string()),
            Instruction::CopyAssignment(p, Address::Constant(Literal::Int(0), Type::Int)),
            Instruction::CopyAssignment(
                p_valid.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ),
            Instruction::Param(p_valid),
            Instruction::Call(None, "print".to_string(), 1),
            Instruction::CopyAssignment(k, Address::Constant(Literal::Int(0), Type::Int)),
            Instruction::CopyAssignment(v.clone(), Address::Constant(Literal::Int(2), Type::Int),),
            Instruction::Param(v),
            Instruction::Call(None, "print".to_string(), 1),
        ]
    );
}
