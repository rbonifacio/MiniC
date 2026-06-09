use crate::ir::ast::{CheckedProgram, CheckedStmt, Statement, Expr, CheckedExpr, Literal, Type};
use crate::ir::tac::{TACProgram, Instruction, Address, Operator};


#[derive(Clone)]
pub struct Environment {
    current_label : usize,
    current_temporary: usize
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_label: 0,
            current_temporary: 0
        }
    }

    fn new_label(&mut self) -> String {
        self.current_label += 1;
        format!("Label{}:", self.current_label)
    }

    fn new_temporary(&mut self) -> String {
        self.current_temporary += 1;
        format!("temp{}", self.current_temporary)
    }
}

fn translate_program(program: CheckedProgram, env: &mut Environment) -> TACProgram {
    let main_fn = program.main_function();
    match main_fn {
        None => unreachable!("[Impossible] program must have a main function"),
        Some(f) => translate_function_body((*f.body).clone(), env),
    }
}

fn translate_function_body(body: CheckedStmt, env: &mut Environment) -> TACProgram {
    if let Statement::Block { seq : stmts } = body.stmt {
        stmts.into_iter().flat_map(|stmt| translate_statement(stmt, env)).collect::<Vec<_>>()
    } else {
        translate_statement(body, env)
    }
}

pub fn translate_statement(statement: CheckedStmt, env: &mut Environment) -> Vec<Instruction> {
    let mut res: Vec<Instruction> = Vec::new();

    match statement.stmt {
        Statement::Block{seq} => {
            seq.into_iter().flat_map(|s| translate_statement(s, env)).collect::<Vec<_>>()
        },
        Statement::Assign { target, value } => {
            if let Expr::Ident(name) = &target.exp {
                let var_type = target.ty.clone();
                let var_address = Address::Variable(name.to_string(), var_type);
                let (expression_address, instructions) = translate_expression(*value, env);
                res.extend(instructions);
                res.push(Instruction::CopyAssignment(var_address, expression_address));
                res
            }
            else {
                todo!()
            }
        },
        Statement::If{cond, then_branch: then_body, else_branch: Some(else_body)} => {
            let label_else = env.new_label();
            let label_end_if = env.new_label();
            let mut instructions = translate_conditional(*cond, env, label_else.clone());
            instructions.extend(translate_statement(*then_body, env));
            instructions.push(Instruction::JMP(label_end_if.clone()));
            instructions.push(Instruction::Label(label_else));
            instructions.extend(translate_statement(*else_body, env));
            instructions.push(Instruction::Label(label_end_if));
            instructions
        },
        _ => todo!()
    }
}

fn translate_expression(expression: CheckedExpr, env: &mut Environment) -> (Address, Vec<Instruction>) {
    match expression.exp {
        Expr::Literal(value) => {
            (Address::Constant(value, expression.ty), vec![])
        },
        Expr::Ident(name) => {
          (Address::Variable(name.to_string(), expression.ty), vec![])
        },
        // Boolean Expressions. 'and' and 'or' implement a short circuit semantics.
        Expr::Not(exp) => {
            let (addr, mut instructions) = translate_expression(*exp, env);
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            instructions.push(Instruction::ConditionalJMPFalse(addr, label_false.clone()));
            instructions.push(Instruction::CopyAssignment(temp.clone(), Address::Constant(Literal::Bool(false), Type::Bool)));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.push(Instruction::CopyAssignment(temp.clone(), Address::Constant(Literal::Bool(true), Type::Bool)));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        }
        Expr::Or(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let label_true = env.new_label();
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMPFalse(l_addr, label_false.clone()));
            instructions.push(Instruction::JMP(label_true.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.extend(r_instructions);
            instructions.push(Instruction::ConditionalJMP(r_addr, label_true.clone()));
            instructions.push(Instruction::CopyAssignment(temp.clone(), Address::Constant(Literal::Bool(false), Type::Bool)));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_true));
            instructions.push(Instruction::CopyAssignment(temp.clone(), Address::Constant(Literal::Bool(true), Type::Bool)));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        },
        Expr::And(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMPFalse(l_addr, label_false.clone()));
            instructions.extend(r_instructions);
            instructions.push(Instruction::ConditionalJMPFalse(r_addr, label_false.clone()));
            instructions.push(Instruction::CopyAssignment(temp.clone(), Address::Constant(Literal::Bool(true), Type::Bool)));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.push(Instruction::CopyAssignment(temp.clone(), Address::Constant(Literal::Bool(false), Type::Bool)));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        },
        // Arithmetic Expressions
        Expr::Add(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let mut instructions = [l_instructions, r_instructions].concat();
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::BinaryAssignment(Operator::Add, temp.clone(), l_addr, r_addr));
            (temp, instructions)
        }
        _ => todo!()
    }
}

fn translate_conditional(expression: CheckedExpr, env: &mut Environment, false_label: String) -> Vec<Instruction> {
    match expression.exp {
        Expr::Literal(Literal::Bool(true)) => vec![],
        Expr::Literal(Literal::Bool(false)) => vec![Instruction::JMP(false_label)],
        Expr::Ident(name) => {
            let addr = Address::Variable(name.to_string(), expression.ty);
            let instructions = vec![Instruction::ConditionalJMPFalse(addr, false_label)];
            instructions
        },
        // AND: short-circuit — if left is false jump immediately; only evaluate right if left is true
        Expr::And(left, right) => {
            let mut instructions = translate_conditional(*left, env, false_label.clone());
            instructions.extend(translate_conditional(*right, env, false_label));
            instructions
        },
        // OR: short-circuit — if left is true skip right entirely; only evaluate right if left is false
        Expr::Or(left, right) => {
            let label_skip_right = env.new_label();
            let (l_addr, mut instructions) = translate_expression(*left, env);
            instructions.push(Instruction::ConditionalJMP(l_addr, label_skip_right.clone()));
            instructions.extend(translate_conditional(*right, env, false_label));
            instructions.push(Instruction::Label(label_skip_right));
            instructions
        },
        // NOT: jump to false_label when the inner expression is true
        Expr::Not(expr) => {
            let (addr, mut instructions) = translate_expression(*expr, env);
            instructions.push(Instruction::ConditionalJMP(addr, false_label));
            instructions
        },
        // Relational operators: jump to false_label using the negated operator
        Expr::Lt(left, right) => translate_relational(*left, *right, Operator::GTE, false_label, env),
        Expr::Le(left, right) => translate_relational(*left, *right, Operator::GT,  false_label, env),
        Expr::Gt(left, right) => translate_relational(*left, *right, Operator::LTE, false_label, env),
        Expr::Ge(left, right) => translate_relational(*left, *right, Operator::LT,  false_label, env),
        Expr::Eq(left, right) => translate_relational(*left, *right, Operator::NE,  false_label, env),
        Expr::Ne(left, right) => translate_relational(*left, *right, Operator::EQ,  false_label, env),
        // Fallback for plain boolean identifiers/literals
        _ => {
            let (addr, mut instructions) = translate_expression(expression, env);
            instructions.push(Instruction::ConditionalJMPFalse(addr, false_label));
            instructions
        }
    }
}

fn translate_relational(left: CheckedExpr, right: CheckedExpr, op: Operator, false_label: String, env: &mut Environment) -> Vec<Instruction> {
    let (l_addr, l_instructions) = translate_expression(left, env);
    let (r_addr, r_instructions) = translate_expression(right, env);
    let mut instructions = l_instructions;
    instructions.extend(r_instructions);
    instructions.push(Instruction::ConditionalJMPRelational(op, l_addr, r_addr, false_label));
    instructions
}
