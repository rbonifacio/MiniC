//! Integration tests for the MiniC TAC code generator.

use mini_c::codegen::tac_code_gen::{translate_statement, Environment};
use mini_c::ir::ast::{
    CheckedExpr, CheckedStmt, Expr, ExprD, Literal, Statement, StatementD, Type,
};
use mini_c::ir::tac::{Address, Instruction, Operator};

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

fn mul(left: CheckedExpr, right: CheckedExpr) -> CheckedExpr {
    ExprD {
        exp: Expr::Mul(Box::new(left), Box::new(right)),
        ty: Type::Int,
    }
}

fn int_lit(value: i64) -> CheckedExpr {
    ExprD {
        exp: Expr::Literal(Literal::Int(value)),
        ty: Type::Int,
    }
}

fn fn_int_int() -> Type {
    Type::Fun(vec![Type::Int], Box::new(Type::Int))
}

fn lambda_double() -> CheckedExpr {
    let param_name = "x".to_string();
    let body = StatementD {
        stmt: Statement::Return(Some(Box::new(mul(int_var(&param_name), int_lit(2))))),
        ty: Type::Int,
    };
    ExprD {
        exp: Expr::Lambda {
            params: vec![(param_name, Type::Int)],
            return_tipo: Type::Int,
            crp: Box::new(body),
        },
        ty: fn_int_int(),
    }
}

fn call_expr(callee: CheckedExpr, args: Vec<CheckedExpr>, ty: Type) -> CheckedExpr {
    ExprD {
        exp: Expr::CallExpr {
            chmd: Box::new(callee),
            args,
        },
        ty,
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
                Operator::LT,
                x.clone(),
                y.clone(),
                "Label1:".to_string()
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

#[test]
fn test_lambda_assignment_and_indirect_call() {
    let double_ty = fn_int_int();
    let stmt = StatementD {
        stmt: Statement::Block {
            seq: vec![
                StatementD {
                    stmt: Statement::Assign {
                        target: Box::new(ExprD {
                            exp: Expr::Ident("double".to_string()),
                            ty: double_ty.clone(),
                        }),
                        value: Box::new(lambda_double()),
                    },
                    ty: Type::Unit,
                },
                StatementD {
                    stmt: Statement::Assign {
                        target: Box::new(ExprD {
                            exp: Expr::Ident("result".to_string()),
                            ty: Type::Int,
                        }),
                        value: Box::new(call_expr(
                            ExprD {
                                exp: Expr::Ident("double".to_string()),
                                ty: double_ty.clone(),
                            },
                            vec![int_lit(21)],
                            Type::Int,
                        )),
                    },
                    ty: Type::Unit,
                },
            ],
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::JMP("Label1:".to_string()),
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
                Address::Variable("double".to_string(), double_ty.clone()),
                Address::FunctionLabel("lambda_0".to_string())
            ),
            Instruction::Param(Address::Constant(Literal::Int(21), Type::Int)),
            Instruction::CallIndirect(
                Some(Address::Temporary("temp2".to_string(), Type::Int)),
                Address::Variable("double".to_string(), double_ty),
                1
            ),
            Instruction::CopyAssignment(
                Address::Variable("result".to_string(), Type::Int),
                Address::Temporary("temp2".to_string(), Type::Int)
            ),
        ]
    );
}

#[test]
fn test_lambda_declaration_generates_function_label_and_body() {
    let double_ty = fn_int_int();

    let stmt = StatementD {
        stmt: Statement::Decl {
            name: "double".to_string(),
            ty: double_ty.clone(),
            init: Some(Box::new(lambda_double())),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::JMP("Label1:".to_string()),
            Instruction::Label("lambda_0".to_string()),
            Instruction::BinaryAssignment(
                Operator::Mul,
                Address::Temporary("temp1".to_string(), Type::Int),
                Address::Variable("x".to_string(), Type::Int),
                Address::Constant(Literal::Int(2), Type::Int),
            ),
            Instruction::Return(Some(Address::Temporary("temp1".to_string(), Type::Int))),
            Instruction::Label("Label1:".to_string()),
            Instruction::CopyAssignment(
                Address::Variable("double".to_string(), double_ty),
                Address::FunctionLabel("lambda_0".to_string()),
            ),
        ]
    );
}

#[test]
fn test_named_call_fallback_to_indirect_call() {
    let stmt = StatementD {
        stmt: Statement::Call {
            name: "local_func".to_string(),
            args: vec![int_lit(5)],
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    assert_eq!(
        instructions,
        vec![
            Instruction::Param(Address::Constant(Literal::Int(5), Type::Int)),
            Instruction::CallIndirect(
                None,
                Address::Variable(
                    "local_func".to_string(),
                    Type::Fun(vec![Type::Int], Box::new(Type::Any))
                ),
                1
            ),
        ]
    );
}