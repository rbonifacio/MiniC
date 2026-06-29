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
        Instruction::ConditionalJMPRelational(Operator::GTE, x.clone(), y.clone(), "Label1:".to_string()),
        Instruction::BinaryAssignment(Operator::Add, temp.clone(), x.clone(), y.clone()),
        Instruction::CopyAssignment(z.clone(), temp),
        Instruction::JMP("Label2:".to_string()),
        Instruction::Label("Label1:".to_string()),
        Instruction::CopyAssignment(z, x),
        Instruction::Label("Label2:".to_string()),
    ]);
}

#[test]
fn test_pointer_assignment() {

    let p_deref = ExprD {
        exp: Expr::Deref(Box::new(ExprD { exp: Expr::Ident("p".to_string()), ty: Type::Pointer(Box::new(Type::Int)) })),
        ty: Type::Int,
    };
    
    let one = ExprD {
        exp: Expr::Literal(Literal::Int(1)),
        ty: Type::Int,
    };
    
    let add_expr = ExprD {
        exp: Expr::Add(Box::new(p_deref.clone()), Box::new(one)),
        ty: Type::Int,
    };
    
    let stmt = StatementD {
        stmt: Statement::Assign {
            target: Box::new(p_deref),
            value: Box::new(add_expr),
        },
        ty: Type::Unit,
    };
    
    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);
    
    let p = Address::Variable("p".to_string(), Type::Pointer(Box::new(Type::Int)));
    let temp1 = Address::Temporary("temp1".to_string(), Type::Int);
    let temp2 = Address::Temporary("temp2".to_string(), Type::Int);
    
    assert_eq!(instructions, vec![
        Instruction::DerefRead(temp1.clone(), p.clone()),
        Instruction::BinaryAssignment(Operator::Add, temp2.clone(), temp1, Address::Constant(Literal::Int(1), Type::Int)),
        Instruction::DerefWrite(p, temp2),
    ]);
}
