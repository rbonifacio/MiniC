//! Three-Address Code (TAC) generation from a type-checked MiniC program.
//!
//! # Overview
//!
//! This module lowers the *annotated* AST produced by the type checker into a
//! flat list of [`Instruction`]s — the Three-Address Code intermediate
//! representation defined in [`crate::ir::tac`]. Each TAC instruction refers to
//! at most three addresses (a result and up to two operands); complex
//! expressions are decomposed into a sequence of simple instructions whose
//! intermediate results are held in fresh *temporaries* (`temp1`, `temp2`, …).
//!
//! # Two translation contexts for booleans
//!
//! Boolean-valued expressions are translated differently depending on how they
//! are used:
//!
//! * [`translate_conditional`] — the boolean *drives control flow* (the
//!   condition of an `if`, `while`, or `assert`). The result is expressed as
//!   conditional jumps to a *true* or *false* label; no temporary is produced.
//! * [`translate_expression`] — the boolean must be *stored as a value* (e.g.
//!   `x = a && b`). The result is materialised into a fresh temporary through
//!   labelled code.
//!
//! # Project 8 tie-in
//!
//! Two MiniC constructs introduced by Project 8 are lowered here:
//!
//! * An `assert e;` statement becomes a conditional jump: if `e` is true,
//!   control falls through; if false, it jumps to a failure block that calls
//!   the `assert_fail` runtime routine.
//! * Each `test "name" { … }` block becomes its own labelled TAC routine
//!   (`test_<name>:`), translated exactly like a function body.

use crate::ir::ast::{
    CheckedExpr, CheckedFunDecl, CheckedProgram, CheckedStmt, Expr, Literal, Statement, StatementD,
    Type,
};
use crate::ir::tac::{Address, Instruction, Operator, TACProgram};

/// Code-generation state: monotonic counters for fresh labels and temporaries.
#[derive(Clone)]
pub struct Environment {
    current_label: usize,
    current_temporary: usize,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_label: 0,
            current_temporary: 0,
        }
    }

    fn new_label(&mut self) -> String {
        self.current_label += 1;
        format!("L{}", self.current_label)
    }

    fn new_temporary(&mut self) -> String {
        self.current_temporary += 1;
        format!("temp{}", self.current_temporary)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Turn a printable test name into a label-safe suffix.
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Generate TAC for a whole program: every function, followed by every test.
///
/// Functions are emitted first (each prefixed with a `name:` label), then each
/// `test` block becomes a `test_<name>:` routine. This is the Project 8 tie-in:
/// tests are first-class code that the generator lowers just like functions.
pub fn generate_tac(program: &CheckedProgram) -> TACProgram {
    let mut env = Environment::new();
    let mut instructions = Vec::new();

    for function in &program.functions {
        instructions.extend(translate_function(function.clone(), &mut env));
    }

    for test in &program.tests {
        instructions.push(Instruction::Label(format!("test_{}", sanitize(&test.name))));
        instructions.extend(translate_statement(*test.body.clone(), &mut env));
    }

    instructions
}

fn translate_function(function: CheckedFunDecl, env: &mut Environment) -> Vec<Instruction> {
    let body = *function.body;
    let mut instructions = match body.stmt {
        Statement::Block { seq } => seq
            .into_iter()
            .flat_map(|stmt| translate_statement(stmt, env))
            .collect::<Vec<_>>(),
        stmt => translate_statement(
            StatementD {
                stmt,
                ty: body.ty,
            },
            env,
        ),
    };
    instructions.insert(0, Instruction::Label(function.name.clone()));
    instructions
}

pub fn translate_statement(statement: CheckedStmt, env: &mut Environment) -> Vec<Instruction> {
    match statement.stmt {
        Statement::Block { seq } => seq
            .into_iter()
            .flat_map(|s| translate_statement(s, env))
            .collect::<Vec<_>>(),

        Statement::Decl { name, ty, init } => {
            let var = Address::Variable(name, ty);
            let (addr, mut instructions) = translate_expression(*init, env);
            instructions.push(Instruction::CopyAssignment(var, addr));
            instructions
        }

        Statement::Assign { target, value } => {
            if let Expr::Ident(name) = &target.exp {
                let var = Address::Variable(name.to_string(), target.ty.clone());
                let (addr, mut instructions) = translate_expression(*value, env);
                instructions.push(Instruction::CopyAssignment(var, addr));
                instructions
            } else {
                // Assignment through an index target (`a[i] = e`) is out of scope.
                todo!("assignment target other than a plain identifier")
            }
        }

        Statement::Call { name, args } => {
            let addresses_and_instructions = args
                .into_iter()
                .map(|expr| translate_expression(expr, env))
                .collect::<Vec<_>>();
            let mut instructions =
                addresses_and_instructions
                    .iter()
                    .fold(vec![], |mut acc, (_, inst)| {
                        acc.extend(inst.clone());
                        acc
                    });
            for (addr, _) in &addresses_and_instructions {
                instructions.push(Instruction::Param(addr.clone()));
            }
            instructions.push(Instruction::Call(
                None,
                name,
                addresses_and_instructions.len(),
            ));
            instructions
        }

        Statement::If {
            cond,
            then_branch,
            else_branch: Some(else_branch),
        } => {
            let label_then = env.new_label();
            let label_else = env.new_label();
            let label_end = env.new_label();
            let mut instructions =
                translate_conditional(*cond, env, label_then.clone(), label_else.clone());
            instructions.push(Instruction::Label(label_then));
            instructions.extend(translate_statement(*then_branch, env));
            instructions.push(Instruction::JMP(label_end.clone()));
            instructions.push(Instruction::Label(label_else));
            instructions.extend(translate_statement(*else_branch, env));
            instructions.push(Instruction::Label(label_end));
            instructions
        }

        Statement::If {
            cond,
            then_branch,
            else_branch: None,
        } => {
            let label_then = env.new_label();
            let label_end = env.new_label();
            let mut instructions =
                translate_conditional(*cond, env, label_then.clone(), label_end.clone());
            instructions.push(Instruction::Label(label_then));
            instructions.extend(translate_statement(*then_branch, env));
            instructions.push(Instruction::Label(label_end));
            instructions
        }

        Statement::While { cond, body } => {
            let label_start = env.new_label();
            let label_body = env.new_label();
            let label_end = env.new_label();
            let mut instructions = vec![Instruction::Label(label_start.clone())];
            instructions.extend(translate_conditional(
                *cond,
                env,
                label_body.clone(),
                label_end.clone(),
            ));
            instructions.push(Instruction::Label(label_body));
            instructions.extend(translate_statement(*body, env));
            instructions.push(Instruction::JMP(label_start));
            instructions.push(Instruction::Label(label_end));
            instructions
        }

        Statement::Return(opt) => match opt {
            Some(expr) => {
                let (addr, mut instructions) = translate_expression(*expr, env);
                instructions.push(Instruction::Return(Some(addr)));
                instructions
            }
            None => vec![Instruction::Return(None)],
        },

        // Project 8: `assert e;` — fall through when true, jump to a failure
        // block that calls the `assert_fail` runtime routine when false.
        Statement::Assert(expr) => {
            let label_ok = env.new_label();
            let label_fail = env.new_label();
            let mut instructions =
                translate_conditional(*expr, env, label_ok.clone(), label_fail.clone());
            instructions.push(Instruction::Label(label_fail));
            instructions.push(Instruction::Param(Address::Constant(
                Literal::Str("assertion failed".to_string()),
                Type::Str,
            )));
            instructions.push(Instruction::Call(None, "assert_fail".to_string(), 1));
            instructions.push(Instruction::Label(label_ok));
            instructions
        }
    }
}

fn translate_expression(
    expression: CheckedExpr,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    let result_ty = expression.ty.clone();
    match expression.exp {
        Expr::Literal(value) => (Address::Constant(value, result_ty), vec![]),

        Expr::Ident(name) => (Address::Variable(name, result_ty), vec![]),

        // Arithmetic
        Expr::Add(left, right) => translate_binary(Operator::Add, *left, *right, result_ty, env),
        Expr::Sub(left, right) => translate_binary(Operator::Sub, *left, *right, result_ty, env),
        Expr::Mul(left, right) => translate_binary(Operator::Mul, *left, *right, result_ty, env),
        Expr::Div(left, right) => translate_binary(Operator::Div, *left, *right, result_ty, env),

        Expr::Neg(inner) => {
            let (addr, mut instructions) = translate_expression(*inner, env);
            let temp = Address::Temporary(env.new_temporary(), result_ty);
            instructions.push(Instruction::UnaryAssignment(Operator::Neg, temp.clone(), addr));
            (temp, instructions)
        }

        // Relational operators as a *value*: materialise a boolean temporary.
        Expr::Lt(left, right) => translate_relational_value(Operator::LT, *left, *right, env),
        Expr::Le(left, right) => translate_relational_value(Operator::LTE, *left, *right, env),
        Expr::Gt(left, right) => translate_relational_value(Operator::GT, *left, *right, env),
        Expr::Ge(left, right) => translate_relational_value(Operator::GTE, *left, *right, env),
        Expr::Eq(left, right) => translate_relational_value(Operator::EQ, *left, *right, env),
        Expr::Ne(left, right) => translate_relational_value(Operator::NE, *left, *right, env),

        // Boolean expressions as a value. `&&`/`||` keep short-circuit semantics.
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

        // Function call as an expression: emit params, then `t := call f, n`.
        Expr::Call { name, args } => {
            let mut instructions = Vec::new();
            let mut arg_addrs = Vec::new();
            for arg in args {
                let (addr, arg_instructions) = translate_expression(arg, env);
                instructions.extend(arg_instructions);
                arg_addrs.push(addr);
            }
            for addr in &arg_addrs {
                instructions.push(Instruction::Param(addr.clone()));
            }
            let temp = Address::Temporary(env.new_temporary(), result_ty);
            instructions.push(Instruction::Call(Some(temp.clone()), name, arg_addrs.len()));
            (temp, instructions)
        }

        // Arrays are out of scope for this milestone.
        Expr::ArrayLit(_) | Expr::Index { .. } => {
            todo!("array literals and indexing are not yet lowered to TAC")
        }
    }
}

/// Lower a binary arithmetic operation into `t := a <op> b`.
fn translate_binary(
    op: Operator,
    left: CheckedExpr,
    right: CheckedExpr,
    result_ty: Type,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    let (l_addr, l_instructions) = translate_expression(left, env);
    let (r_addr, r_instructions) = translate_expression(right, env);
    let mut instructions = [l_instructions, r_instructions].concat();
    let temp = Address::Temporary(env.new_temporary(), result_ty);
    instructions.push(Instruction::BinaryAssignment(op, temp.clone(), l_addr, r_addr));
    (temp, instructions)
}

/// Materialise a relational comparison into a boolean temporary via a
/// conditional relational jump.
fn translate_relational_value(
    op: Operator,
    left: CheckedExpr,
    right: CheckedExpr,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    let (l_addr, l_instructions) = translate_expression(left, env);
    let (r_addr, r_instructions) = translate_expression(right, env);
    let label_true = env.new_label();
    let label_exit = env.new_label();
    let temp = Address::Temporary(env.new_temporary(), Type::Bool);
    let mut instructions = l_instructions;
    instructions.extend(r_instructions);
    instructions.push(Instruction::ConditionalJMPRelational(
        op,
        l_addr,
        r_addr,
        label_true.clone(),
    ));
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

/// Translate a boolean expression used as a *condition*: emit jumps so that
/// control reaches `true_label` when the expression is true and `false_label`
/// when it is false. No temporary is produced.
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
            let addr = Address::Variable(name, expression.ty);
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
