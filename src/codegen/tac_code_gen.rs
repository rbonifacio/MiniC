//! Geração de Three-Address Code (TAC) a partir da AST tipada.
//!
//! Cada função traduz um nível da árvore (programa → função → statement → expressão)
//! e devolve instruções TAC lineares com saltos condicionais quando necessário.

// Tipos da AST já verificada pelo type checker.
use crate::ir::ast::{CheckedProgram, CheckedFunDecl, CheckedStmt, Statement, Expr, CheckedExpr, Literal, Type};
// Representação das instruções TAC e dos endereços (variável, constante, temporário).
use crate::ir::tac::{TACProgram, Instruction, Address, Operator};

/// Estado compartilhado durante a tradução de uma função.
/// Garante nomes únicos para labels de salto e variáveis temporárias.
#[derive(Clone)]
pub struct Environment {
    current_label: usize,      // próximo número de label a alocar (Label1:, Label2:, ...)
    current_temporary: usize,  // próximo número de temporário a alocar (temp1, temp2, ...)
}

impl Environment {
    /// Inicia o ambiente com contadores zerados.
    pub fn new() -> Self {
        Self {
            current_label: 0,
            current_temporary: 0,
        }
    }

    /// Aloca um novo label de salto (ex.: `"Label3:"`).
    fn new_label(&mut self) -> String {
        self.current_label += 1;
        format!("Label{}:", self.current_label)
    }

    /// Aloca um novo temporário (ex.: `"temp2"`).
    fn new_temporary(&mut self) -> String {
        self.current_temporary += 1;
        format!("temp{}", self.current_temporary)
    }
}

/// Ponto de entrada público: traduz um programa inteiro em TAC.
/// Hoje gera TAC apenas da função `main`.
pub fn translate_program(program: &CheckedProgram) -> TACProgram {
    let mut env = Environment::new();
    translate_program_with_env(program, &mut env)
}

/// Localiza `main` no programa e delega a tradução da função.
fn translate_program_with_env(program: &CheckedProgram, env: &mut Environment) -> TACProgram {
    let main_fn = program.main_function();
    match main_fn {
        // O type checker já garante que `main` existe; este caso não deve ocorrer.
        None => unreachable!("[Impossible] program must have a main function"),
        Some(f) => translate_function(f.clone(), env),
    }
}

/// Traduz o corpo de uma função e prefixa um label com o nome da função.
fn translate_function(function: CheckedFunDecl, env: &mut Environment) -> TACProgram {
    let mut instructions =
        // Corpo costuma ser um bloco `{ stmt; stmt; ... }`.
        if let Statement::Block { seq: stmts } = function.body.stmt {
            // Traduz cada statement e concatena as listas de instruções.
            stmts
                .into_iter()
                .flat_map(|stmt| translate_statement(stmt, env))
                .collect::<Vec<_>>()
        } else {
            // Gramática também permite um único statement sem chaves.
            translate_statement(*function.body, env)
        };
    // Marca o início da função (destino de `call` e saltos).
    instructions.insert(0, Instruction::Label(function.name.clone()));
    instructions
}

/// Traduz um statement da AST em zero ou mais instruções TAC.
pub fn translate_statement(statement: CheckedStmt, env: &mut Environment) -> Vec<Instruction> {
    // Acumulador reutilizado nos casos que montam `res` incrementalmente.
    let mut res: Vec<Instruction> = Vec::new();

    match statement.stmt {
        // `{ s1; s2; ... }` → traduz cada filho em sequência.
        Statement::Block { seq } => seq
            .into_iter()
            .flat_map(|s| translate_statement(s, env))
            .collect::<Vec<_>>(),

        // `x = expr` — por enquanto só atribuição a identificador simples.
        Statement::Assign { target, value } => {
            if let Expr::Ident(name) = &target.exp {
                let var_type = target.ty.clone();
                let var_address = Address::Variable(name.to_string(), var_type);
                // Primeiro avalia o lado direito; depois copia o resultado para `x`.
                let (expression_address, instructions) = translate_expression(*value, env);
                res.extend(instructions);
                res.push(Instruction::CopyAssignment(var_address, expression_address));
                res
            } else {
                // Atribuição indexada (`arr[i] = v`) ainda não implementada.
                todo!()
            }
        },

        // Chamada como statement: `foo(a, b);` — descarta o retorno (`Call(None, ...)`).
        Statement::Call { name, args } => {
            // Para cada argumento: endereço do valor + instruções para calculá-lo.
            let addresses_and_instructions = args
                .into_iter()
                .map(|expr| translate_expression(expr, env))
                .collect::<Vec<_>>();
            // Junta todas as instruções de avaliação dos argumentos.
            let mut instructions = addresses_and_instructions
                .iter()
                .fold(vec![], |mut acc, (_, inst)| {
                    acc.extend(inst.clone());
                    acc
                });
            // Convenção de chamada: um `param` por argumento, na ordem.
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

        // Declaração com init: `int x = expr;` → avalia `expr` e copia para `x`.
        Statement::Decl { name, ty, init } => {
            let (expression_address, instructions) = translate_expression(*init, env);
            res.extend(instructions);
            res.push(Instruction::CopyAssignment(
                Address::Variable(name, ty),
                expression_address,
            ));
            res
        },

        // `if (cond) then else` — três labels: then, else e fim do if.
        Statement::If {
            cond,
            then_branch: then_body,
            else_branch: Some(else_body),
        } => {
            let label_then = env.new_label();
            let label_else = env.new_label();
            let label_end_if = env.new_label();
            // Gera saltos conforme `cond`; verdadeiro → then, falso → else.
            let mut instructions =
                translate_conditional(*cond, env, label_then.clone(), label_else.clone());
            instructions.push(Instruction::Label(label_then));
            instructions.extend(translate_statement(*then_body, env));
            instructions.push(Instruction::JMP(label_end_if.clone())); // pula o else
            instructions.push(Instruction::Label(label_else));
            instructions.extend(translate_statement(*else_body, env));
            instructions.push(Instruction::Label(label_end_if));
            instructions
        },

        // `for (init; cond; update) body`
        // vira em instructions do tipo:
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
            let label_test = env.new_label(); // topo do loop (teste de condição)
            let label_exit = env.new_label(); // saída do loop
            let mut instructions = Vec::new();

            // Init roda uma única vez antes do loop.
            if let Some(init_stmt) = init {
                instructions.extend(translate_statement(*init_stmt, env));
            }

            // Label do topo do loop (teste de condição)
            instructions.push(Instruction::Label(label_test.clone()));

            // Se houver condição, falso salta para `label_exit`; verdadeiro cai no corpo.
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

            // Update roda após o corpo, antes de voltar ao teste.
            if let Some(u) = update {
                instructions.extend(translate_statement(*u, env));
            }

            instructions.push(Instruction::JMP(label_test)); // próxima iteração
            instructions.push(Instruction::Label(label_exit));
            instructions
        },

        // While, Return, If sem else, etc. — ainda não implementados.
        _ => todo!(),
    }
}

/// Traduz uma expressão.
/// Retorna `(endereço_do_resultado, instruções_auxiliares)`.
/// O endereço pode ser constante, variável ou temporário onde o valor ficou armazenado.
fn translate_expression(
    expression: CheckedExpr,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    match expression.exp {
        // Literal: endereço imediato, sem instruções extras.
        Expr::Literal(value) => (Address::Constant(value, expression.ty), vec![]),

        // Variável: referência direta, sem instruções extras.
        Expr::Ident(name) => (Address::Variable(name.to_string(), expression.ty), vec![]),

        // `!e` — avalia `e` como condição e materializa o booleano negado em um temp.
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

        // `a || b` — short-circuit: se `a` é true, não avalia `b`.
        Expr::Or(left, right) => {
            let (l_addr, l_instructions) = translate_expression(*left, env);
            let (r_addr, r_instructions) = translate_expression(*right, env);
            let label_true = env.new_label();
            let label_false = env.new_label();
            let label_exit = env.new_label();
            let temp = Address::Temporary(env.new_temporary(), Type::Bool);
            let mut instructions = l_instructions;
            instructions.push(Instruction::ConditionalJMPFalse(l_addr, label_false.clone()));
            instructions.push(Instruction::JMP(label_true.clone())); // `a` true → resultado true
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

        // `a && b` — short-circuit: se `a` é false, não avalia `b`.
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

        // `a + b` — avalia operandos e grava a soma em um temporário.
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

        // Sub, Mul, Call, Index, etc. — ainda não implementados.
        _ => todo!(),
    }
}

/// Traduz uma expressão usada como *condição* de controle de fluxo.
/// Emite saltos para `true_label` (condição verdadeira) ou `false_label` (falsa),
/// em vez de materializar um booleano em temporário.
fn translate_conditional(
    expression: CheckedExpr,
    env: &mut Environment,
    true_label: String,
    false_label: String,
) -> Vec<Instruction> {
    match expression.exp {
        // Constantes booleanas: salto direto, sem código de teste.
        Expr::Literal(Literal::Bool(true)) => vec![Instruction::JMP(true_label)],
        Expr::Literal(Literal::Bool(false)) => vec![Instruction::JMP(false_label)],

        // Variável bool: se true → true_label; senão cai no JMP para false_label.
        Expr::Ident(name) => {
            let addr = Address::Variable(name.to_string(), expression.ty);
            vec![
                Instruction::ConditionalJMP(addr, true_label),
                Instruction::JMP(false_label),
            ]
        }

        // `a && b`: se `a` falha vai direto para false; senão testa `b`.
        Expr::And(left, right) => {
            let label_right = env.new_label();
            let mut instructions =
                translate_conditional(*left, env, label_right.clone(), false_label.clone());
            instructions.push(Instruction::Label(label_right));
            instructions.extend(translate_conditional(*right, env, true_label, false_label));
            instructions
        }

        // `a || b`: se `a` passa vai direto para true; senão testa `b`.
        Expr::Or(left, right) => {
            let label_right = env.new_label();
            let mut instructions =
                translate_conditional(*left, env, true_label.clone(), label_right.clone());
            instructions.push(Instruction::Label(label_right));
            instructions.extend(translate_conditional(*right, env, true_label, false_label));
            instructions
        }

        // `!e` inverte os destinos true/false.
        Expr::Not(expr) => translate_conditional(*expr, env, false_label, true_label),

        // Comparações: saltam para true_label se a relação vale; senão JMP false_label.
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

        // Expressão geral: avalia para um endereço e trata como truthy/falsy.
        _ => {
            let (addr, mut instructions) = translate_expression(expression, env);
            instructions.push(Instruction::ConditionalJMP(addr, true_label));
            instructions.push(Instruction::JMP(false_label));
            instructions
        }
    }
}

/// Avalia dois operandos e emite salto relacional:
/// `if lhs op rhs goto true_label` seguido de `goto false_label`.
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
