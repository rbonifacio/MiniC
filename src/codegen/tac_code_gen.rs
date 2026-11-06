use crate::ir::ast::{CheckedProgram, CheckedFunDecl, CheckedStmt, Statement, Expr, CheckedExpr, Literal, Type};
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

// A program is the concatenation of the code of all its functions.
pub fn translate_program(program: CheckedProgram, env: &mut Environment) -> TACProgram {
    program.functions.into_iter().fold(vec![], |mut acc, f| {acc.extend(translate_function(f, env)); acc})
}

// A function starts with a label carrying its name, followed by one
// identity statement per parameter (as in Jimple), and then its body:
//
//   f:
//   a = @parameter0
//   b = @parameter1
//   <body>
pub fn translate_function(function: CheckedFunDecl, env: &mut Environment) -> TACProgram {
    let mut instructions = vec![Instruction::Label(function.name)];
    for (i, (name, ty)) in function.params.into_iter().enumerate() {
        let parameter = Address::Identity(format!("parameter{}", i), ty.clone());
        instructions.push(Instruction::CopyAssignment(Address::Variable(name, ty), parameter));
    }
    instructions.extend(translate_statement(*function.body, env));
    instructions
}

pub fn translate_statement(statement: CheckedStmt, env: &mut Environment) -> Vec<Instruction> {
    let mut res: Vec<Instruction> = Vec::new();

    match statement.stmt {
        Statement::Block{seq} => {
            seq.into_iter().flat_map(|s| translate_statement(s, env)).collect::<Vec<_>>()
        },
        Statement::Decl { name, ty, init } => {
            let (init_addr, instrs) = translate_expression(*init, env);
            res.extend(instrs);
            res.push(Instruction::CopyAssignment(Address::Variable(name, ty), init_addr));
            res
        },
        Statement::Assign { target, value } => {
            match target.exp {
                Expr::Ident(name) => {
                    let var_addr = Address::Variable(name.to_string(), target.ty);
                    let (val_addr, instrs) = translate_expression(*value, env);
                    res.extend(instrs);
                    res.push(Instruction::CopyAssignment(var_addr, val_addr));
                    res
                },
                Expr::Deref(inner) => {
                    let (ptr_addr, ptr_instrs) = translate_expression(*inner, env);
                    let (val_addr, val_instrs) = translate_expression(*value, env);
                    res.extend(ptr_instrs);
                    res.extend(val_instrs);
                    res.push(Instruction::DerefWrite(ptr_addr, val_addr)); 
                    res
                },

                _ => todo!() 
            }
        },
        Statement::Call{name, args} => {
            // As a statement, the returned value (if any) is discarded.
            let (mut instructions, n) = translate_arguments(args, env);
            instructions.push(Instruction::Call(None, name, n));
            instructions
        }
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
        Statement::If{cond, then_branch: then_body, else_branch: None} => {
            let label_end_if = env.new_label();
            res.extend(translate_conditional(*cond, env, label_end_if.clone()));
            res.extend(translate_statement(*then_body, env));
            res.push(Instruction::Label(label_end_if));
            res
        },
        Statement::While { cond, body } => {
            let label_loop = env.new_label();
            let label_end  = env.new_label();
            res.push(Instruction::Label(label_loop.clone()));
            res.extend(translate_conditional(*cond, env, label_end.clone()));
            res.extend(translate_statement(*body, env));
            res.push(Instruction::JMP(label_loop));
            res.push(Instruction::Label(label_end));
            res
        },
        Statement::Return(Some(value)) => {
            let (addr, mut instructions) = translate_expression(*value, env);
            instructions.push(Instruction::Return(Some(addr)));
            instructions
        },
        Statement::Return(None) => vec![Instruction::Return(None)],
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
        Expr::Add(left, right) => translate_binary(Operator::Add, *left, *right, expression.ty, env),
        // Relational Expressions
        Expr::Lt(left, right) => translate_binary(Operator::LT,  *left, *right, expression.ty, env),
        Expr::Le(left, right) => translate_binary(Operator::LTE, *left, *right, expression.ty, env),
        Expr::Gt(left, right) => translate_binary(Operator::GT,  *left, *right, expression.ty, env),
        Expr::Ge(left, right) => translate_binary(Operator::GTE, *left, *right, expression.ty, env),
        Expr::Eq(left, right) => translate_binary(Operator::EQ,  *left, *right, expression.ty, env),
        Expr::Ne(left, right) => translate_binary(Operator::NE,  *left, *right, expression.ty, env),
        Expr::AddrOf(inner) => {
            let (inner_addr, mut instructions) = translate_expression(*inner, env);
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::AddressOf(temp.clone(), inner_addr));
            (temp, instructions)
        },
        Expr::Deref(inner) => {
            let (inner_addr, mut instructions) = translate_expression(*inner, env);
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::DerefRead(temp.clone(), inner_addr));
            (temp, instructions)
        },
        // As an expression, the returned value goes into a fresh temporary.
        Expr::Call{name, args} => {
            let (mut instructions, n) = translate_arguments(args, env);
            let temp = Address::Temporary(env.new_temporary(), expression.ty);
            instructions.push(Instruction::Call(Some(temp.clone()), name, n));
            (temp, instructions)
        },
        _ => todo!()
    }
}

// Evaluates the condition and jumps to 'false_label' when it does not hold.
// When it holds, execution falls through to the next instruction.
// Short-circuit evaluation of 'and', 'or', and 'not' is handled by
// translate_expression.
fn translate_conditional(expression: CheckedExpr, env: &mut Environment, false_label: String) -> Vec<Instruction> {
    let (addr, mut instructions) = translate_expression(expression, env);
    instructions.push(Instruction::ConditionalJMPFalse(addr, false_label));
    instructions
}

fn translate_binary(op: Operator, left: CheckedExpr, right: CheckedExpr, ty: Type, env: &mut Environment) -> (Address, Vec<Instruction>) {
    let (l_addr, l_instructions) = translate_expression(left, env);
    let (r_addr, r_instructions) = translate_expression(right, env);
    let mut instructions = [l_instructions, r_instructions].concat();
    let temp = Address::Temporary(env.new_temporary(), ty);
    instructions.push(Instruction::BinaryAssignment(op, temp.clone(), l_addr, r_addr));
    (temp, instructions)
}

// Evaluates all arguments (left to right) and then emits one 'param'
// per argument. Returns the instructions and the number of arguments,
// which the caller uses to build the 'call' instruction.
fn translate_arguments(args: Vec<CheckedExpr>, env: &mut Environment) -> (Vec<Instruction>, usize) {
    // addresses_and_instructions :: [(Address, [Instruction])]
    let addresses_and_instructions = args.into_iter().map(|expr| translate_expression(expr, env)).collect::<Vec<_>>();
    let mut instructions = addresses_and_instructions.iter().fold(vec![], |mut acc, (_, inst)| {acc.extend(inst.clone()); acc});

    // includes a 'param' instruction for
    // every address built from the arguments.
    for (addr, _) in &addresses_and_instructions {
        instructions.push(Instruction::Param(addr.clone()));
    }
    (instructions, addresses_and_instructions.len())
}
