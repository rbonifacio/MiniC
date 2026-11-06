//! Integration tests for the MiniC TAC code generator.

use mini_c::ir::ast::{CheckedExpr, CheckedStmt, ExprD, Expr, FunDecl, Literal, Program, Statement, StatementD, Type};
use mini_c::ir::tac::{Address, Instruction, Operator};
use mini_c::codegen::tac_code_gen::{Environment, translate_program, translate_statement};

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
//   temp1 = x < y
//   ifFalse temp1 goto Label1:
//   temp2 = x + y
//   z = temp2
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
    let cond = Address::Temporary("temp1".to_string(), Type::Bool);
    let temp = Address::Temporary("temp2".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::BinaryAssignment(Operator::LT, cond.clone(), x.clone(), y.clone()),
        Instruction::ConditionalJMPFalse(cond, "Label1:".to_string()),
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

#[test]
fn test_pointer_address_of() {

    let addr_of_expr = ExprD {
        exp: Expr::AddrOf(Box::new(ExprD { 
            exp: Expr::Ident("x".to_string()), 
            ty: Type::Int 
        })),
        ty: Type::Pointer(Box::new(Type::Int)),
    };
    
    let stmt = StatementD {
        stmt: Statement::Assign {
            target: Box::new(ExprD { 
                exp: Expr::Ident("y".to_string()), 
                ty: Type::Pointer(Box::new(Type::Int)) 
            }),
            value: Box::new(addr_of_expr),
        },
        ty: Type::Unit,
    };
    
    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);
    
    let x = Address::Variable("x".to_string(), Type::Int);
    let y = Address::Variable("y".to_string(), Type::Pointer(Box::new(Type::Int)));
    let temp1 = Address::Temporary("temp1".to_string(), Type::Pointer(Box::new(Type::Int)));
    
    assert_eq!(instructions, vec![
        Instruction::AddressOf(temp1.clone(), x),
        Instruction::CopyAssignment(y, temp1),
    ]);
}

// Fixture: while (i < n) { i = i + 1; }
//
// Expected TAC:
//   Label1:
//   temp1 = i < n
//   ifFalse temp1 goto Label2:
//   temp2 = i + 1
//   i = temp2
//   goto Label1:
//   Label2:
#[test]
fn test_while_with_relational_condition() {
    let one = ExprD { exp: Expr::Literal(Literal::Int(1)), ty: Type::Int };
    let stmt = StatementD {
        stmt: Statement::While {
            cond: Box::new(lt(int_var("i"), int_var("n"))),
            body: Box::new(assign("i", add(int_var("i"), one))),
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let i    = Address::Variable("i".to_string(), Type::Int);
    let n    = Address::Variable("n".to_string(), Type::Int);
    let c1   = Address::Constant(Literal::Int(1), Type::Int);
    let cond = Address::Temporary("temp1".to_string(), Type::Bool);
    let temp = Address::Temporary("temp2".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::Label("Label1:".to_string()),
        Instruction::BinaryAssignment(Operator::LT, cond.clone(), i.clone(), n),
        Instruction::ConditionalJMPFalse(cond, "Label2:".to_string()),
        Instruction::BinaryAssignment(Operator::Add, temp.clone(), i.clone(), c1),
        Instruction::CopyAssignment(i, temp),
        Instruction::JMP("Label1:".to_string()),
        Instruction::Label("Label2:".to_string()),
    ]);
}

// Fixture: if (x < y) { z = x; }
//
// Expected TAC:
//   temp1 = x < y
//   ifFalse temp1 goto Label1:
//   z = x
//   Label1:
#[test]
fn test_if_without_else() {
    let stmt = StatementD {
        stmt: Statement::If {
            cond:        Box::new(lt(int_var("x"), int_var("y"))),
            then_branch: Box::new(assign("z", int_var("x"))),
            else_branch: None,
        },
        ty: Type::Unit,
    };

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let x    = Address::Variable("x".to_string(), Type::Int);
    let y    = Address::Variable("y".to_string(), Type::Int);
    let z    = Address::Variable("z".to_string(), Type::Int);
    let cond = Address::Temporary("temp1".to_string(), Type::Bool);

    assert_eq!(instructions, vec![
        Instruction::BinaryAssignment(Operator::LT, cond.clone(), x.clone(), y),
        Instruction::ConditionalJMPFalse(cond, "Label1:".to_string()),
        Instruction::CopyAssignment(z, x),
        Instruction::Label("Label1:".to_string()),
    ]);
}

fn call(name: &str, args: Vec<CheckedExpr>) -> CheckedExpr {
    ExprD { exp: Expr::Call { name: name.to_string(), args }, ty: Type::Int }
}

// Fixture: x = f(a + 1, g(b));
//
// Expected TAC (all arguments are evaluated before any 'param' of f):
//   temp1 = a + 1
//   param b
//   temp2 = call g, 1
//   param temp1
//   param temp2
//   temp3 = call f, 2
//   x = temp3
#[test]
fn test_nested_call_expression() {
    let one = ExprD { exp: Expr::Literal(Literal::Int(1)), ty: Type::Int };
    let stmt = assign("x", call("f", vec![
        add(int_var("a"), one),
        call("g", vec![int_var("b")]),
    ]));

    let mut env = Environment::new();
    let instructions = translate_statement(stmt, &mut env);

    let a  = Address::Variable("a".to_string(), Type::Int);
    let b  = Address::Variable("b".to_string(), Type::Int);
    let x  = Address::Variable("x".to_string(), Type::Int);
    let c1 = Address::Constant(Literal::Int(1), Type::Int);
    let t1 = Address::Temporary("temp1".to_string(), Type::Int);
    let t2 = Address::Temporary("temp2".to_string(), Type::Int);
    let t3 = Address::Temporary("temp3".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::BinaryAssignment(Operator::Add, t1.clone(), a, c1),
        Instruction::Param(b),
        Instruction::Call(Some(t2.clone()), "g".to_string(), 1),
        Instruction::Param(t1),
        Instruction::Param(t2),
        Instruction::Call(Some(t3.clone()), "f".to_string(), 2),
        Instruction::CopyAssignment(x, t3),
    ]);
}

fn stmt(s: Statement<Type>) -> CheckedStmt {
    StatementD { stmt: s, ty: Type::Unit }
}

// Fixture:
//   int sum(int a, int b) { return a + b; }
//   void main() { int x = sum(1, 2); return; }
//
// Expected TAC (parameters are received through Jimple-style
// identity statements):
//   sum:
//   a = @parameter0
//   b = @parameter1
//   temp1 = a + b
//   return temp1
//   main:
//   param 1
//   param 2
//   temp2 = call sum, 2
//   x = temp2
//   return
#[test]
fn test_program_with_two_functions() {
    let int_lit = |n| ExprD { exp: Expr::Literal(Literal::Int(n)), ty: Type::Int };
    let sum = FunDecl {
        name: "sum".to_string(),
        params: vec![("a".to_string(), Type::Int), ("b".to_string(), Type::Int)],
        return_type: Type::Int,
        body: Box::new(stmt(Statement::Block { seq: vec![
            stmt(Statement::Return(Some(Box::new(add(int_var("a"), int_var("b")))))),
        ]})),
    };
    let main = FunDecl {
        name: "main".to_string(),
        params: vec![],
        return_type: Type::Unit,
        body: Box::new(stmt(Statement::Block { seq: vec![
            stmt(Statement::Decl {
                name: "x".to_string(),
                ty: Type::Int,
                init: Box::new(call("sum", vec![int_lit(1), int_lit(2)])),
            }),
            stmt(Statement::Return(None)),
        ]})),
    };

    let mut env = Environment::new();
    let instructions = translate_program(Program { functions: vec![sum, main] }, &mut env);

    let a  = Address::Variable("a".to_string(), Type::Int);
    let b  = Address::Variable("b".to_string(), Type::Int);
    let x  = Address::Variable("x".to_string(), Type::Int);
    let t1 = Address::Temporary("temp1".to_string(), Type::Int);
    let t2 = Address::Temporary("temp2".to_string(), Type::Int);

    assert_eq!(instructions, vec![
        Instruction::Label("sum".to_string()),
        Instruction::CopyAssignment(a.clone(), Address::Identity("parameter0".to_string(), Type::Int)),
        Instruction::CopyAssignment(b.clone(), Address::Identity("parameter1".to_string(), Type::Int)),
        Instruction::BinaryAssignment(Operator::Add, t1.clone(), a, b),
        Instruction::Return(Some(t1)),
        Instruction::Label("main".to_string()),
        Instruction::Param(Address::Constant(Literal::Int(1), Type::Int)),
        Instruction::Param(Address::Constant(Literal::Int(2), Type::Int)),
        Instruction::Call(Some(t2.clone()), "sum".to_string(), 2),
        Instruction::CopyAssignment(x, t2),
        Instruction::Return(None),
    ]);
}
