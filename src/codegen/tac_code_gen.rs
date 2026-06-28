use crate::ir::ast::{
    CheckedExpr, CheckedFunDecl, CheckedProgram, CheckedStmt, Expr, ExprD, Literal, Statement, Type,
};
use crate::ir::tac::{Address, Instruction, Operator, TACProgram};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Environment {
    current_label: usize,
    current_temporary: usize,
    current_lambda: usize,
    global_functions: HashSet<String>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_label: 0,
            current_temporary: 0,
            current_lambda: 0,
            global_functions: HashSet::new(),
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

    fn new_function_label(&mut self) -> String {
        let label = format!("lambda_{}", self.current_lambda);
        self.current_lambda += 1;
        label
    }

    fn set_global_functions(&mut self, global_functions: HashSet<String>) {
        self.global_functions = global_functions;
    }

    fn is_global_function(&self, name: &str) -> bool {
        self.global_functions.contains(name)
    }
}

pub fn translate_program(program: CheckedProgram, env: &mut Environment) -> TACProgram {
    let global_functions = program
        .functions
        .iter()
        .map(|function| function.name.clone())
        .collect::<HashSet<_>>();
    env.set_global_functions(global_functions);

    program
        .functions
        .into_iter()
        .flat_map(|function| translate_function(function, env))
        .collect()
}

fn translate_function(function: CheckedFunDecl, env: &mut Environment) -> TACProgram {
    let mut instructions = if let Statement::Block { seq: stmts } = function.body.stmt {
        stmts
            .into_iter()
            .flat_map(|stmt| translate_statement(stmt, env))
            .collect::<Vec<_>>()
    } else {
        translate_statement(*(function.body), env)
    };
    instructions.insert(0, Instruction::Label(function.name.clone()));
    instructions
}

pub fn translate_statement(statement: CheckedStmt, env: &mut Environment) -> Vec<Instruction> {
    let mut res: Vec<Instruction> = Vec::new();

    match statement.stmt {
        Statement::Block { seq } => seq
            .into_iter()
            .flat_map(|s| translate_statement(s, env))
            .collect::<Vec<_>>(),
        Statement::Decl { name, ty, init } => {
            let var_address = Address::Variable(name, ty);

            match init {
                Some(expr) => {
                    let (expression_address, instructions) = translate_expression(*expr, env);

                    res.extend(instructions);
                    res.push(Instruction::CopyAssignment(var_address, expression_address));

                    res
                }

                None => res,
            }
        }
        Statement::Assign { target, value } => {
            if let Expr::Ident(name) = &target.exp {
                let var_type = target.ty.clone();
                let var_address = Address::Variable(name.to_string(), var_type);
                let (expression_address, instructions) = translate_expression(*value, env);
                res.extend(instructions);
                res.push(Instruction::CopyAssignment(var_address, expression_address));
                res
            } else {
                todo!()
            }
        }
        Statement::Call { name, args } => translate_named_call(name, args, None, env),
        Statement::If {
            cond,
            then_branch: then_body,
            else_branch: Some(else_body),
        } => {
            let label_then = env.new_label();
            let label_else = env.new_label();
            let label_end_if = env.new_label();
            let mut instructions =
                translate_conditional(*cond, env, label_then.clone(), label_else.clone());
            instructions.push(Instruction::Label(label_then));
            instructions.extend(translate_statement(*then_body, env));
            instructions.push(Instruction::JMP(label_end_if.clone()));
            instructions.push(Instruction::Label(label_else));
            instructions.extend(translate_statement(*else_body, env));
            instructions.push(Instruction::Label(label_end_if));
            instructions
        }
        Statement::If {
            cond,
            then_branch: then_body,
            else_branch: None,
        } => {
            let label_then = env.new_label();
            let label_end_if = env.new_label();
            let mut instructions =
                translate_conditional(*cond, env, label_then.clone(), label_end_if.clone());
            instructions.push(Instruction::Label(label_then));
            instructions.extend(translate_statement(*then_body, env));
            instructions.push(Instruction::Label(label_end_if));
            instructions
        }
        Statement::Return(value) => match value {
            Some(expr) => {
                let (address, instructions) = translate_expression(*expr, env);
                res.extend(instructions);
                res.push(Instruction::Return(Some(address)));
                res
            }
            None => vec![Instruction::Return(None)],
        },
        _ => todo!(),
    }
}

fn translate_expression(
    expression: CheckedExpr,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    match expression.exp {
        Expr::Literal(value) => (Address::Constant(value, expression.ty), vec![]),
        Expr::Ident(name) => (Address::Variable(name.to_string(), expression.ty), vec![]),
        // Boolean Expressions. 'and' and 'or' implement a short circuit semantics.
        Expr::Not(exp) => {
            let (addr, mut instructions) = translate_expression(*exp, env);
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            instructions.push(Instruction::ConditionalJMPFalse(addr, label_false.clone()));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(false), Type::Bool),
            ));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ));
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
            instructions.push(Instruction::ConditionalJMPFalse(
                l_addr,
                label_false.clone(),
            ));
            instructions.push(Instruction::JMP(label_true.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.extend(r_instructions);
            instructions.push(Instruction::ConditionalJMP(r_addr, label_true.clone()));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(false), Type::Bool),
            ));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_true));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        }
        Expr::And(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMPFalse(
                l_addr,
                label_false.clone(),
            ));
            instructions.extend(r_instructions);
            instructions.push(Instruction::ConditionalJMPFalse(
                r_addr,
                label_false.clone(),
            ));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(false), Type::Bool),
            ));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        }
        // Arithmetic Expressions
        Expr::Add(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let mut instructions = [l_instructions, r_instructions].concat();
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::BinaryAssignment(
                Operator::Add,
                temp.clone(),
                l_addr,
                r_addr,
            ));
            (temp, instructions)
        }
        Expr::Sub(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let mut instructions = [l_instructions, r_instructions].concat();
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::BinaryAssignment(
                Operator::Sub,
                temp.clone(),
                l_addr,
                r_addr,
            ));
            (temp, instructions)
        }
        Expr::Mul(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let mut instructions = [l_instructions, r_instructions].concat();
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::BinaryAssignment(
                Operator::Mul,
                temp.clone(),
                l_addr,
                r_addr,
            ));
            (temp, instructions)
        }
        Expr::Div(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let mut instructions = [l_instructions, r_instructions].concat();
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::BinaryAssignment(
                Operator::Div,
                temp.clone(),
                l_addr,
                r_addr,
            ));
            (temp, instructions)
        }
        Expr::Neg(exp) => {
            let (addr, mut instructions) = translate_expression(*exp, env);
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::UnaryAssignment(
                Operator::Neg,
                temp.clone(),
                addr,
            ));
            (temp, instructions)
        }
        Expr::Lt(_, _)
        | Expr::Le(_, _)
        | Expr::Gt(_, _)
        | Expr::Ge(_, _)
        | Expr::Eq(_, _)
        | Expr::Ne(_, _) => {
            let label_true = env.new_label();
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = translate_conditional(
                ExprD {
                    exp: expression.exp,
                    ty: Type::Bool,
                },
                env,
                label_true.clone(),
                label_false.clone(),
            );
            instructions.push(Instruction::Label(label_true));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(true), Type::Bool),
            ));
            instructions.push(Instruction::JMP(label_exit.clone()));
            instructions.push(Instruction::Label(label_false));
            instructions.push(Instruction::CopyAssignment(
                temp.clone(),
                Address::Constant(Literal::Bool(false), Type::Bool),
            ));
            instructions.push(Instruction::Label(label_exit));
            (temp, instructions)
        }
        Expr::Call { name, args } => {
            let result = Address::Temporary(env.new_temporary(), expression.ty);
            let instructions = translate_named_call(name, args, Some(result.clone()), env);

            (result, instructions)
        }
        Expr::CallExpr { chmd, args } => {
            let (callee_address, mut instructions) = translate_expression(*chmd, env);
            let (arguments, argument_instructions) = translate_call_arguments(args, env);
            instructions.extend(argument_instructions);
            for address in &arguments {
                instructions.push(Instruction::Param(address.clone()));
            }

            let result = Address::Temporary(env.new_temporary(), expression.ty);
            match callee_address {
                Address::FunctionLabel(name) => {
                    instructions.push(Instruction::Call(
                        Some(result.clone()),
                        name,
                        arguments.len(),
                    ));
                }
                other => {
                    instructions.push(Instruction::CallIndirect(
                        Some(result.clone()),
                        other,
                        arguments.len(),
                    ));
                }
            }

            (result, instructions)
        }
        Expr::Lambda {
            params: _,
            return_tipo: _,
            crp,
        } => {
            // Nome da função gerada
            let lambda_label = env.new_function_label();

            // Label para continuar a execução
            let exit_label = env.new_label();

            // Valor produzido pela expressão lambda
            let closure =
                Address::Temporary(env.new_temporary(), expression.ty.clone());

            let mut instructions = Vec::new();

            // pula o corpo da lambda
            instructions.push(Instruction::JMP(exit_label.clone()));

            // início da função
            instructions.push(Instruction::Label(lambda_label.clone()));

            // traduz normalmente o corpo
            instructions.extend(translate_statement(*crp, env));

            // caso o usuário esqueça do return
            if !matches!(
                instructions.last(),
                Some(Instruction::Return(_))
            ) {
                instructions.push(Instruction::Return(None));
            }

            // continua a execução
            instructions.push(Instruction::Label(exit_label));

            // cria a closure capturando o ambiente atual
            instructions.push(
                Instruction::MakeClosure(
                    closure.clone(),
                    lambda_label,
                )
            );

            (closure, instructions)
        }
        _ => todo!(),
    }
}

fn translate_call_arguments(
    args: Vec<CheckedExpr>,
    env: &mut Environment,
) -> (Vec<Address>, Vec<Instruction>) {
    let addresses_and_instructions = args
        .into_iter()
        .map(|expr| translate_expression(expr, env))
        .collect::<Vec<_>>();
    let instructions = addresses_and_instructions
        .iter()
        .fold(vec![], |mut acc, (_, inst)| {
            acc.extend(inst.clone());
            acc
        });
    let addresses = addresses_and_instructions
        .into_iter()
        .map(|(addr, _)| addr)
        .collect::<Vec<_>>();
    (addresses, instructions)
}

fn translate_named_call(
    name: String,
    args: Vec<CheckedExpr>,
    return_target: Option<Address>,
    env: &mut Environment,
) -> Vec<Instruction> {
    let arg_types = args.iter().map(|arg| arg.ty.clone()).collect::<Vec<_>>();
    let (addresses, mut instructions) = translate_call_arguments(args, env);
    for address in &addresses {
        instructions.push(Instruction::Param(address.clone()));
    }
    if env.is_global_function(&name) {
        instructions.push(Instruction::Call(return_target, name, addresses.len()));
    } else {
        let return_type = return_target
            .as_ref()
            .and_then(address_type)
            .unwrap_or(Type::Any);
        let callee = Address::Variable(name, Type::Fun(arg_types, Box::new(return_type)));
        instructions.push(Instruction::CallIndirect(return_target, callee, addresses.len()));
    }
    instructions
}

fn address_type(address: &Address) -> Option<Type> {
    match address {
        Address::Variable(_, ty) => Some(ty.clone()),
        Address::Constant(_, ty) => Some(ty.clone()),
        Address::Temporary(_, ty) => Some(ty.clone()),
        Address::FunctionLabel(_) => None,
    }
}

fn translate_conditional(
    expression: CheckedExpr,
    env: &mut Environment,
    true_label: String,
    false_label: String,
) -> Vec<Instruction> {
    match expression.exp {
        Expr::Literal(Literal::Bool(true)) => vec![Instruction::JMP(true_label)],
        Expr::Literal(Literal::Bool(false)) => vec![Instruction::JMP(false_label)],
        Expr::Ident(name) => {
            let addr = Address::Variable(name.to_string(), expression.ty);
            vec![
                Instruction::ConditionalJMP(addr, true_label),
                Instruction::JMP(false_label),
            ]
        }
        Expr::And(left, right) => {
            let label_right = env.new_label();
            let mut instructions =
                translate_conditional(*left, env, label_right.clone(), false_label.clone());
            instructions.push(Instruction::Label(label_right));
            instructions.extend(translate_conditional(*right, env, true_label, false_label));
            instructions
        }
        Expr::Or(left, right) => {
            let label_right = env.new_label();
            let mut instructions =
                translate_conditional(*left, env, true_label.clone(), label_right.clone());
            instructions.push(Instruction::Label(label_right));
            instructions.extend(translate_conditional(*right, env, true_label, false_label));
            instructions
        }
        Expr::Not(expr) => translate_conditional(*expr, env, false_label, true_label),
        Expr::Lt(left, right) => {
            translate_relational(*left, *right, Operator::LT, true_label, false_label, env)
        }
        Expr::Le(left, right) => {
            translate_relational(*left, *right, Operator::LTE, true_label, false_label, env)
        }
        Expr::Gt(left, right) => {
            translate_relational(*left, *right, Operator::GT, true_label, false_label, env)
        }
        Expr::Ge(left, right) => {
            translate_relational(*left, *right, Operator::GTE, true_label, false_label, env)
        }
        Expr::Eq(left, right) => {
            translate_relational(*left, *right, Operator::EQ, true_label, false_label, env)
        }
        Expr::Ne(left, right) => {
            translate_relational(*left, *right, Operator::NE, true_label, false_label, env)
        }
        _ => {
            let (addr, mut instructions) = translate_expression(expression, env);
            instructions.push(Instruction::ConditionalJMP(addr, true_label));
            instructions.push(Instruction::JMP(false_label));
            instructions
        }
    }
}

fn translate_relational(
    left: CheckedExpr,
    right: CheckedExpr,
    op: Operator,
    true_label: String,
    false_label: String,
    env: &mut Environment,
) -> Vec<Instruction> {
    let (l_addr, l_instructions) = translate_expression(left, env);
    let (r_addr, r_instructions) = translate_expression(right, env);
    let mut instructions = l_instructions;
    instructions.extend(r_instructions);
    instructions.push(Instruction::ConditionalJMPRelational(
        op, l_addr, r_addr, true_label,
    ));
    instructions.push(Instruction::JMP(false_label));
    instructions
}
