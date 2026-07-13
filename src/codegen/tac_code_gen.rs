use crate::ir::ast::{
    CheckedExpr, CheckedFunDecl, CheckedProgram, CheckedStmt, Expr, ExprD, Literal, Statement,
    Type, UDTDecl, UDTKind, UDTMember,
};
use crate::ir::tac::{Address, Instruction, Operator, TACProgram};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Environment {
    current_label: usize,
    current_temporary: usize,
    current_struct_temp: usize,
    current_match_binding: usize,
    type_declarations: Vec<UDTDecl>,
    binding_aliases: HashMap<String, Address>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_label: 0,
            current_temporary: 0,
            current_struct_temp: 0,
            current_match_binding: 0,
            type_declarations: Vec::new(),
            binding_aliases: HashMap::new(),
        }
    }

    fn register_type_declarations(&mut self, type_declarations: Vec<UDTDecl>) {
        self.type_declarations = type_declarations;
    }

    fn new_label(&mut self) -> String {
        self.current_label += 1;
        format!("Label{}:", self.current_label)
    }

    fn new_temporary(&mut self) -> String {
        self.current_temporary += 1;
        format!("temp{}", self.current_temporary)
    }

    fn new_struct_temp(&mut self) -> String {
        self.current_struct_temp += 1;
        format!("_init{}", self.current_struct_temp)
    }

    fn new_match_binding(&mut self) -> String {
        self.current_match_binding += 1;
        format!("_match{}", self.current_match_binding)
    }

    fn alias_for(&self, name: &str) -> Option<Address> {
        self.binding_aliases.get(name).cloned()
    }

    fn push_alias(&mut self, name: String, address: Address) -> Option<Address> {
        self.binding_aliases.insert(name, address)
    }

    fn restore_alias(&mut self, name: String, previous: Option<Address>) {
        if let Some(address) = previous {
            self.binding_aliases.insert(name, address);
        } else {
            self.binding_aliases.remove(&name);
        }
    }

    fn variant_tag_index(&self, enum_name: &str, variant: &str) -> i64 {
        let decl = self
            .type_declarations
            .iter()
            .find(|decl| decl.specifier == UDTKind::Enum && decl.identifier == enum_name)
            .unwrap_or_else(|| unreachable!("checked enum type must be declared"));

        for (i, member) in decl.members.iter().enumerate() {
            if let UDTMember::EnumVariant { name, .. } = member {
                if name == variant {
                    return i as i64;
                }
            }
        }

        unreachable!("checked enum variant must exist")
    }

    fn variant_payload_ty(&self, enum_name: &str, variant: &str) -> Option<Type> {
        self.type_declarations
            .iter()
            .find(|d| d.specifier == UDTKind::Enum && d.identifier == enum_name)?
            .members
            .iter()
            .find_map(|m| match m {
                UDTMember::EnumVariant { name, ty } if name == variant => ty.clone(),
                _ => None,
            })
    }

    fn struct_fields(&self, struct_name: &str) -> Vec<(String, Type)> {
        self.type_declarations
            .iter()
            .find(|d| d.specifier == UDTKind::Struct && d.identifier == struct_name)
            .unwrap_or_else(|| unreachable!("checked struct type must be declared"))
            .members
            .iter()
            .filter_map(|m| match m {
                UDTMember::Field(field) => Some((field.name.clone(), field.ty.clone())),
                _ => None,
            })
            .collect()
    }

    fn enum_payload_types(&self, enum_name: &str) -> Vec<Type> {
        let mut payload_types = Vec::new();
        if let Some(decl) = self
            .type_declarations
            .iter()
            .find(|d| d.specifier == UDTKind::Enum && d.identifier == enum_name)
        {
            for member in &decl.members {
                if let UDTMember::EnumVariant { ty: Some(ty), .. } = member {
                    if !payload_types.contains(ty) {
                        payload_types.push(ty.clone());
                    }
                }
            }
        }
        payload_types
    }
}

pub fn translate_program(program: CheckedProgram, env: &mut Environment) -> TACProgram {
    env.register_type_declarations(program.type_declarations.clone());
    let main_fn = program.main_function();
    match main_fn {
        None => unreachable!("[Impossible] program must have a main function"),
        Some(f) => translate_function(f.clone(), env),
    }
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
        Statement::Decl { name, ty, init } => lower_copy_to_prefix(&name, &ty, *init, env),
        Statement::Assign { target, value } => {
            let var_address = translate_lvalue(*target, env);
            lower_copy_to_address(var_address, *value, env)
        }
        Statement::Call { name, args } => {
            // addresses_and_instructions :: [(Address, [Instruction])]
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

            // includes a 'param' instruction to the
            // every addresses built from the arguments.
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
            then_branch: then_body,
            else_branch: Some(else_body),
        } => {
            let label_else = env.new_label();
            let label_end_if = env.new_label();
            let mut instructions = translate_conditional_false(*cond, env, label_else.clone());
            instructions.extend(translate_statement(*then_body, env));
            instructions.push(Instruction::JMP(label_end_if.clone()));
            instructions.push(Instruction::Label(label_else));
            instructions.extend(translate_statement(*else_body, env));
            instructions.push(Instruction::Label(label_end_if));
            instructions
        }
        Statement::Match { target, arms } => {
            let mut res = Vec::new();
            let end_label = env.new_label();

            let enum_name = match &target.ty {
                Type::Enum(name) => name.clone(),
                _ => unreachable!("match target must be enum type"),
            };

            let (target_addr, target_insts) = translate_expression(*target, env);
            res.extend(target_insts);

            let tag_var = match &target_addr {
                Address::Variable(name, _) => name.clone(),
                _ => unreachable!("match target must be a variable"),
            };

            for arm in arms.into_iter() {
                let next_label = env.new_label();
                let ordinal = env.variant_tag_index(&enum_name, &arm.variant);
                let tag_addr = Address::Variable(format!("{}.tag", tag_var), Type::Int);
                let ordinal_addr = Address::Constant(Literal::Int(ordinal), Type::Int);

                res.push(Instruction::ConditionalJMPRelational(
                    Operator::NE,
                    tag_addr,
                    ordinal_addr,
                    next_label.clone(),
                ));

                let active_alias = if let Some(binding) = arm.binding {
                    let payload_ty = env
                        .variant_payload_ty(&enum_name, &arm.variant)
                        .expect("variant with binding must have payload type");
                    let binding_addr =
                        Address::Temporary(env.new_match_binding(), payload_ty.clone());
                    let payload_addr = Address::Variable(format!("{}.payload", tag_var), payload_ty);
                    res.push(Instruction::CopyAssignment(
                        binding_addr.clone(),
                        payload_addr,
                    ));
                    let binding_name = binding.clone();
                    let previous = env.push_alias(binding, binding_addr);
                    Some((binding_name, previous))
                } else {
                    None
                };

                res.extend(translate_statement(*arm.body, env));

                if let Some((binding, previous)) = active_alias {
                    env.restore_alias(binding, previous);
                }

                res.push(Instruction::JMP(end_label.clone()));
                res.push(Instruction::Label(next_label));
            }

            res.push(Instruction::Label(end_label));
            res
        }
        _ => todo!(),
    }
}

fn translate_lvalue(target: CheckedExpr, env: &mut Environment) -> Address {
    match target.exp {
        Expr::Ident(name) => env
            .alias_for(&name)
            .unwrap_or_else(|| Address::Variable(name, target.ty)),
        Expr::Member { base, member } => translate_member_address(*base, member, target.ty, env),
        _ => todo!(),
    }
}

fn translate_member_address(
    base: CheckedExpr,
    member: String,
    member_ty: Type,
    env: &mut Environment,
) -> Address {
    let (base_address, instructions) = translate_expression(base, env);
    debug_assert!(instructions.is_empty());
    Address::Variable(format!("{}.{}", address_prefix(&base_address), member), member_ty)
}

fn address_prefix(address: &Address) -> String {
    match address {
        Address::Variable(name, _) | Address::Temporary(name, _) => name.clone(),
        Address::Constant(_, _) => unreachable!("UDT value cannot be a scalar constant"),
    }
}

fn lower_copy_to_address(
    destination: Address,
    source: CheckedExpr,
    env: &mut Environment,
) -> Vec<Instruction> {
    match &destination {
        Address::Variable(name, ty) | Address::Temporary(name, ty) => {
            lower_copy_to_prefix(name, ty, source, env)
        }
        Address::Constant(_, _) => unreachable!("cannot assign to constant address"),
    }
}

fn lower_copy_to_prefix(
    destination_prefix: &str,
    destination_ty: &Type,
    source: CheckedExpr,
    env: &mut Environment,
) -> Vec<Instruction> {
    match destination_ty {
        Type::Struct(struct_name) => lower_struct_copy(destination_prefix, struct_name, source, env),
        Type::Enum(enum_name) => lower_enum_copy(destination_prefix, enum_name, source, env),
        _ => {
            let (source_address, mut instructions) = translate_expression(source, env);
            instructions.push(Instruction::CopyAssignment(
                Address::Variable(destination_prefix.to_string(), destination_ty.clone()),
                source_address,
            ));
            instructions
        }
    }
}

fn lower_struct_copy(
    destination_prefix: &str,
    struct_name: &str,
    source: CheckedExpr,
    env: &mut Environment,
) -> Vec<Instruction> {
    match source.exp {
        Expr::Cast { expr, .. } => lower_copy_to_prefix(destination_prefix, &source.ty, *expr, env),
        Expr::Init { fields } => {
            let mut instructions = Vec::new();
            for (field_name, field_expr_opt) in fields {
                let field_expr = field_expr_opt.expect("struct field must have a value");
                let field_ty = field_expr.ty.clone();
                instructions.extend(lower_copy_to_prefix(
                    &format!("{}.{}", destination_prefix, field_name),
                    &field_ty,
                    field_expr,
                    env,
                ));
            }
            instructions
        }
        other => {
            let (source_address, mut instructions) = translate_expression(
                ExprD {
                    exp: other,
                    ty: source.ty,
                },
                env,
            );
            let source_prefix = address_prefix(&source_address);
            for (field_name, field_ty) in env.struct_fields(struct_name) {
                instructions.extend(lower_component_copy(
                    &format!("{}.{}", destination_prefix, field_name),
                    &format!("{}.{}", source_prefix, field_name),
                    &field_ty,
                    env,
                ));
            }
            instructions
        }
    }
}

fn lower_enum_copy(
    destination_prefix: &str,
    enum_name: &str,
    source: CheckedExpr,
    env: &mut Environment,
) -> Vec<Instruction> {
    match source.exp {
        Expr::Cast { expr, .. } => lower_copy_to_prefix(destination_prefix, &source.ty, *expr, env),
        Expr::EnumVariant {
            enum_name: source_enum_name,
            variant,
            payload,
        } => {
            let source_enum_name = source_enum_name.unwrap_or_else(|| enum_name.to_string());
            let ordinal = env.variant_tag_index(&source_enum_name, &variant);
            let mut instructions = vec![Instruction::CopyAssignment(
                Address::Variable(format!("{}.tag", destination_prefix), Type::Int),
                Address::Constant(Literal::Int(ordinal), Type::Int),
            )];
            if let Some(payload_expr) = payload {
                let payload_ty = payload_expr.ty.clone();
                instructions.extend(lower_copy_to_prefix(
                    &format!("{}.payload", destination_prefix),
                    &payload_ty,
                    *payload_expr,
                    env,
                ));
            }
            instructions
        }
        other => {
            let (source_address, mut instructions) = translate_expression(
                ExprD {
                    exp: other,
                    ty: source.ty,
                },
                env,
            );
            let source_prefix = address_prefix(&source_address);
            instructions.push(Instruction::CopyAssignment(
                Address::Variable(format!("{}.tag", destination_prefix), Type::Int),
                Address::Variable(format!("{}.tag", source_prefix), Type::Int),
            ));
            for payload_ty in env.enum_payload_types(enum_name) {
                instructions.extend(lower_component_copy(
                    &format!("{}.payload", destination_prefix),
                    &format!("{}.payload", source_prefix),
                    &payload_ty,
                    env,
                ));
            }
            instructions
        }
    }
}

fn lower_component_copy(
    destination_prefix: &str,
    source_prefix: &str,
    ty: &Type,
    env: &mut Environment,
) -> Vec<Instruction> {
    match ty {
        Type::Struct(struct_name) => {
            let mut instructions = Vec::new();
            for (field_name, field_ty) in env.struct_fields(struct_name) {
                instructions.extend(lower_component_copy(
                    &format!("{}.{}", destination_prefix, field_name),
                    &format!("{}.{}", source_prefix, field_name),
                    &field_ty,
                    env,
                ));
            }
            instructions
        }
        Type::Enum(enum_name) => {
            let mut instructions = vec![Instruction::CopyAssignment(
                Address::Variable(format!("{}.tag", destination_prefix), Type::Int),
                Address::Variable(format!("{}.tag", source_prefix), Type::Int),
            )];
            for payload_ty in env.enum_payload_types(enum_name) {
                instructions.extend(lower_component_copy(
                    &format!("{}.payload", destination_prefix),
                    &format!("{}.payload", source_prefix),
                    &payload_ty,
                    env,
                ));
            }
            instructions
        }
        _ => vec![Instruction::CopyAssignment(
            Address::Variable(destination_prefix.to_string(), ty.clone()),
            Address::Variable(source_prefix.to_string(), ty.clone()),
        )],
    }
}

fn translate_conditional_false(
    expression: CheckedExpr,
    env: &mut Environment,
    false_label: String,
) -> Vec<Instruction> {
    match expression.exp {
        Expr::Literal(Literal::Bool(true)) => vec![],
        Expr::Literal(Literal::Bool(false)) => vec![Instruction::JMP(false_label)],
        Expr::Ident(name) => {
            let addr = env
                .alias_for(&name)
                .unwrap_or_else(|| Address::Variable(name.to_string(), expression.ty));
            vec![Instruction::ConditionalJMPFalse(addr, false_label)]
        }
        Expr::Lt(left, right) => {
            translate_relational_false(*left, *right, Operator::GTE, false_label, env)
        }
        Expr::Le(left, right) => {
            translate_relational_false(*left, *right, Operator::GT, false_label, env)
        }
        Expr::Gt(left, right) => {
            translate_relational_false(*left, *right, Operator::LTE, false_label, env)
        }
        Expr::Ge(left, right) => {
            translate_relational_false(*left, *right, Operator::LT, false_label, env)
        }
        Expr::Eq(left, right) => {
            translate_relational_false(*left, *right, Operator::NE, false_label, env)
        }
        Expr::Ne(left, right) => {
            translate_relational_false(*left, *right, Operator::EQ, false_label, env)
        }
        _ => {
            let (addr, mut instructions) = translate_expression(
                ExprD {
                    exp: expression.exp,
                    ty: expression.ty,
                },
                env,
            );
            instructions.push(Instruction::ConditionalJMPFalse(addr, false_label));
            instructions
        }
    }
}

fn translate_expression(
    expression: CheckedExpr,
    env: &mut Environment,
) -> (Address, Vec<Instruction>) {
    match expression.exp {
        Expr::Literal(value) => (Address::Constant(value, expression.ty), vec![]),
        Expr::Ident(name) => (
            env.alias_for(&name)
                .unwrap_or_else(|| Address::Variable(name.to_string(), expression.ty)),
            vec![],
        ),
        Expr::Member { base, member } => (
            translate_member_address(*base, member, expression.ty, env),
            vec![],
        ),
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
        Expr::Init { fields } => {
            let temp_name = env.new_struct_temp();
            let instructions = lower_copy_to_prefix(
                &temp_name,
                &expression.ty,
                ExprD {
                    exp: Expr::Init { fields },
                    ty: expression.ty.clone(),
                },
                env,
            );
            (Address::Variable(temp_name, expression.ty), instructions)
        }
        Expr::EnumVariant {
            enum_name,
            variant,
            payload,
        } => {
            let temp_name = env.new_struct_temp();
            let instructions = lower_copy_to_prefix(
                &temp_name,
                &expression.ty,
                ExprD {
                    exp: Expr::EnumVariant {
                        enum_name,
                        variant,
                        payload,
                    },
                    ty: expression.ty.clone(),
                },
                env,
            );
            (Address::Variable(temp_name, expression.ty), instructions)
        }
        Expr::Cast { ty: _cast_ty, expr } => {
            let (addr, insts) = translate_expression(*expr, env);
            let cast_addr = match addr {
                Address::Constant(lit, _) => Address::Constant(lit, expression.ty),
                Address::Variable(name, _) => Address::Variable(name, expression.ty),
                Address::Temporary(name, _) => Address::Temporary(name, expression.ty),
            };
            (cast_addr, insts)
        }
        _ => todo!(),
    }
}

fn translate_relational_false(
    left: CheckedExpr,
    right: CheckedExpr,
    op: Operator,
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
        false_label,
    ));
    instructions
}
