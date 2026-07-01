//! Three-Address Code (TAC) generation from the typed AST.
//!
//! Each function translates one level of the tree (program → function → statement → expression)
//! and produces linear TAC instructions with conditional jumps when needed.

// Types from the AST already verified by the type checker.
use crate::ir::ast::{CheckedProgram, CheckedFunDecl, CheckedStmt, Statement, Expr, CheckedExpr, Literal, Type};
// TAC instruction and address representation (variable, constant, temporary).
use crate::ir::tac::{TACProgram, Instruction, Address, Operator};

/// Shared state during function translation.
/// Ensures unique names for jump labels and temporary variables.
#[derive(Clone)]
pub struct Environment {
    current_label: usize,      // next label number to allocate (Label1:, Label2:, ...)
    current_temporary: usize,  // next temporary number to allocate (temp1, temp2, ...)
}

impl Environment {
    /// Creates an environment with zeroed counters.
    pub fn new() -> Self {
        Self {
            current_label: 0,
            current_temporary: 0,
        }
    }

    /// Allocates a new jump label (e.g. `"Label3:"`).
    fn new_label(&mut self) -> String {
        self.current_label += 1;
        format!("Label{}:", self.current_label)
    }

    /// Allocates a new temporary (e.g. `"temp2"`).
    fn new_temporary(&mut self) -> String {
        self.current_temporary += 1;
        format!("temp{}", self.current_temporary)
    }
}

/// Public entry point: translates an entire program into TAC.
/// Currently generates TAC for the `main` function only.
pub fn translate_program(program: &CheckedProgram) -> TACProgram {
    let mut env = Environment::new();
    translate_program_with_env(program, &mut env)
}

/// Locates `main` in the program and delegates function translation.
fn translate_program_with_env(program: &CheckedProgram, env: &mut Environment) -> TACProgram {
    let main_fn = program.main_function();
    match main_fn {
        // The type checker already guarantees `main` exists; this case should not occur.
        None => unreachable!("[Impossible] program must have a main function"),
        Some(f) => translate_function(f.clone(), env),
    }
}

/// Translates a function body and prefixes a label with the function name.
fn translate_function(function: CheckedFunDecl, env: &mut Environment) -> TACProgram {
    let mut instructions =
        // Body is usually a block `{ stmt; stmt; ... }`.
        if let Statement::Block { seq: stmts } = function.body.stmt {
            // Translate each statement and concatenate instruction lists.
            stmts
                .into_iter()
                .flat_map(|stmt| translate_statement(stmt, env))
                .collect::<Vec<_>>()
        } else {
            // Grammar also allows a single statement without braces.
            translate_statement(*function.body, env)
        };
    // Mark function entry (target of `call` and jumps).
    instructions.insert(0, Instruction::Label(function.name.clone()));
    instructions
}

/// Translates an AST statement into zero or more TAC instructions.
pub fn translate_statement(statement: CheckedStmt, env: &mut Environment) -> Vec<Instruction> {
    // Accumulator reused in cases that build `res` incrementally.
    let mut res: Vec<Instruction> = Vec::new();

    match statement.stmt {
        // `{ s1; s2; ... }` → translate each child in sequence.
        Statement::Block { seq } => seq
            .into_iter()
            .flat_map(|s| translate_statement(s, env))
            .collect::<Vec<_>>(),

        // `x = expr` — simple identifier assignment only for now.
        Statement::Assign { target, value } => {
            if let Expr::Ident(name) = &target.exp {
                let var_type = target.ty.clone();
                let var_address = Address::Variable(name.to_string(), var_type);
                // Evaluate the right-hand side first; then copy the result to `x`.
                let (expression_address, instructions) = translate_expression(*value, env);
                res.extend(instructions);
                res.push(Instruction::CopyAssignment(var_address, expression_address));
                res
            } else {
                // Indexed assignment (`arr[i] = v`) not yet implemented.
                todo!()
            }
        },

        // Call as statement: `foo(a, b);` — discard return value (`Call(None, ...)`).
        Statement::Call { name, args } => {
            // For each argument: value address + instructions to compute it.
            let addresses_and_instructions = args
                .into_iter()
                .map(|expr| translate_expression(expr, env))
                .collect::<Vec<_>>();
            // Concatenate all argument evaluation instructions.
            let mut instructions = addresses_and_instructions
                .iter()
                .fold(vec![], |mut acc, (_, inst)| {
                    acc.extend(inst.clone());
                    acc
                });
            // Calling convention: one `param` per argument, in order.
            for (addr, _) in &addresses_and_instructions {
                instructions.push(Instruction::Param(addr.clone()));
            }
            instructions.push(Instruction::Call(
                None,
                name,
                addresses_and_instructions.len(),
            ));
            instructions
        },

        // Declaration with init: `int x = expr;` → evaluate `expr` and copy to `x`.
        Statement::Decl { name, ty, init } => {
            let (expression_address, instructions) = translate_expression(*init, env);
            res.extend(instructions);
            res.push(Instruction::CopyAssignment(
                Address::Variable(name, ty),
                expression_address,
            ));
            res
        },

        // `if (cond) then else` — three labels: then, else, and end of if.
        Statement::If {
            cond,
            then_branch: then_body,
            else_branch: Some(else_body),
        } => {
            let label_then = env.new_label();
            let label_else = env.new_label();
            let label_end_if = env.new_label();
            // Emit jumps according to `cond`; true → then, false → else.
            let mut instructions =
                translate_conditional(*cond, env, label_then.clone(), label_else.clone());
            instructions.push(Instruction::Label(label_then));
            instructions.extend(translate_statement(*then_body, env));
            instructions.push(Instruction::JMP(label_end_if.clone())); // skip else
            instructions.push(Instruction::Label(label_else));
            instructions.extend(translate_statement(*else_body, env));
            instructions.push(Instruction::Label(label_end_if));
            instructions
        },

        // `for (init; cond; update) body`
        // becomes instructions of the form:
        // Label1:
        // if cond goto Label2:
        // goto Label3:
        // Label2:
        // body
        // update
        // goto Label1:
        // Label3:
        Statement::For {
            init,
            cond,
            update,
            body,
        } => {
            let label_test = env.new_label(); // loop head (condition test)
            let label_exit = env.new_label(); // loop exit
            let mut instructions = Vec::new();

            // Init runs once before the loop.
            if let Some(init_stmt) = init {
                instructions.extend(translate_statement(*init_stmt, env));
            }

            // Label at loop head (condition test)
            instructions.push(Instruction::Label(label_test.clone()));

            // If there is a condition, false jumps to `label_exit`; true falls into body.
            if let Some(c) = cond {
                let label_body = env.new_label();
                instructions.extend(translate_conditional(
                    *c,
                    env,
                    label_body.clone(),
                    label_exit.clone(),
                ));
                instructions.push(Instruction::Label(label_body));
            }

            instructions.extend(translate_statement(*body, env));

            // Update runs after the body, before jumping back to the test.
            if let Some(u) = update {
                instructions.extend(translate_statement(*u, env));
            }

            instructions.push(Instruction::JMP(label_test)); // next iteration
            instructions.push(Instruction::Label(label_exit));
            instructions
        },

        // While, Return, If without else, etc. — not yet implemented.
        _ => todo!(),
    }
}

/// Translates an expression.
/// Returns `(result_address, auxiliary_instructions)`.
/// The address may be a constant, variable, or temporary where the value was stored.
fn translate_expression(
    expression: CheckedExpr,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    match expression.exp {
        // Literal: immediate address, no extra instructions.
        Expr::Literal(value) => (Address::Constant(value, expression.ty), vec![]),

        // Variable: direct reference, no extra instructions.
        Expr::Ident(name) => (Address::Variable(name.to_string(), expression.ty), vec![]),

        // `!e` — evaluate `e` as a condition and materialize the negated boolean in a temp.
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

        // `a || b` — short-circuit: if `a` is true, do not evaluate `b`.
        Expr::Or(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let label_true = env.new_label();
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMPFalse(l_addr, label_false.clone()));
            instructions.push(Instruction::JMP(label_true.clone())); // `a` true → result true
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

        // `a && b` — short-circuit: if `a` is false, do not evaluate `b`.
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

        // `a + b` — evaluate operands and store the sum in a temporary.
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

        // Sub, Mul, Call, Index, etc. — not yet implemented.
        _ => todo!(),
    }
}

/// Translates an expression used as a control-flow *condition*.
/// Emits jumps to `true_label` (condition true) or `false_label` (false),
/// instead of materializing a boolean in a temporary.
fn translate_conditional(
    expression: CheckedExpr,
    env: &mut Environment,
    true_label: String,
    false_label: String,
) -> Vec<Instruction> {
    match expression.exp {
        // Boolean constants: direct jump, no test code.
        Expr::Literal(Literal::Bool(true)) => vec![Instruction::JMP(true_label)],
        Expr::Literal(Literal::Bool(false)) => vec![Instruction::JMP(false_label)],

        // Boolean variable: if true → true_label; otherwise fall through to JMP false_label.
        Expr::Ident(name) => {
            let addr = Address::Variable(name.to_string(), expression.ty);
            vec![
                Instruction::ConditionalJMP(addr, true_label),
                Instruction::JMP(false_label),
            ]
        }

        // `a && b`: if `a` fails go directly to false; otherwise test `b`.
        Expr::And(left, right) => {
            let label_right = env.new_label();
            let mut instructions =
                translate_conditional(*left, env, label_right.clone(), false_label.clone());
            instructions.push(Instruction::Label(label_right));
            instructions.extend(translate_conditional(*right, env, true_label, false_label));
            instructions
        }

        // `a || b`: if `a` succeeds go directly to true; otherwise test `b`.
        Expr::Or(left, right) => {
            let label_right = env.new_label();
            let mut instructions =
                translate_conditional(*left, env, true_label.clone(), label_right.clone());
            instructions.push(Instruction::Label(label_right));
            instructions.extend(translate_conditional(*right, env, true_label, false_label));
            instructions
        }

        // `!e` swaps true/false destinations.
        Expr::Not(expr) => translate_conditional(*expr, env, false_label, true_label),

        // Comparisons: jump to true_label if relation holds; otherwise JMP false_label.
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

        // General expression: evaluate to an address and treat as truthy/falsy.
        _ => {
            let (addr, mut instructions) = translate_expression(expression, env);
            instructions.push(Instruction::ConditionalJMP(addr, true_label));
            instructions.push(Instruction::JMP(false_label));
            instructions
        }
    }
}

/// Evaluates two operands and emits a relational jump:
/// `if lhs op rhs goto true_label` followed by `goto false_label`.
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
        op,
        l_addr,
        r_addr,
        true_label,
    ));
    instructions.push(Instruction::JMP(false_label));
    instructions
}
