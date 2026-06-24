//! Integration tests for the MiniC parser.

use nom::combinator::all_consuming;
use mini_c::ir::ast::{Expr, ExprD, Literal, Statement, Type};
use mini_c::parser::{
    assignment, expression, fun_decl, identifier, literal,
    literals::{
        boolean_literal, float_literal, integer_literal, string_literal, Literal as ParserLiteral,
    },
    statement,
};
use mini_c::parser::functions::type_name;

// --- Literals ---

#[test]
fn test_integer_positive() {
    assert_eq!(integer_literal("42"), Ok(("", 42)));
    assert_eq!(integer_literal("0"), Ok(("", 0)));
    assert_eq!(integer_literal("12345"), Ok(("", 12345)));
}

#[test]
fn test_integer_negative() {
    assert_eq!(integer_literal("-17"), Ok(("", -17)));
    assert_eq!(integer_literal("-0"), Ok(("", 0)));
}

#[test]
fn test_integer_reject() {
    assert!(integer_literal("abc").is_err());
    assert!(integer_literal("12.34").is_err());
}

#[test]
fn test_float() {
    assert_eq!(float_literal("3.14"), Ok(("", 3.14)));
    assert_eq!(float_literal("0.5"), Ok(("", 0.5)));
    assert_eq!(float_literal("-0.25"), Ok(("", -0.25)));
}

#[test]
fn test_string_simple() {
    assert_eq!(string_literal(r#""hello""#), Ok(("", "hello".to_string())));
    assert_eq!(string_literal(r#""""#), Ok(("", "".to_string())));
}

#[test]
fn test_string_escapes() {
    assert_eq!(string_literal(r#""a\"b""#), Ok(("", r#"a"b"#.to_string())));
    assert_eq!(
        string_literal(r#""line1\nline2""#),
        Ok(("", "line1\nline2".to_string()))
    );
    assert_eq!(
        string_literal(r#""tab\there""#),
        Ok(("", "tab\there".to_string()))
    );
}

#[test]
fn test_boolean() {
    assert_eq!(boolean_literal("true"), Ok(("", true)));
    assert_eq!(boolean_literal("false"), Ok(("", false)));
}

#[test]
fn test_boolean_reject() {
    assert!(boolean_literal("True").is_err());
    assert!(boolean_literal("1").is_err());
}

#[test]
fn test_literal_combined() {
    assert_eq!(literal("42"), Ok(("", ParserLiteral::Int(42))));
    assert_eq!(literal("3.14"), Ok(("", ParserLiteral::Float(3.14))));
    assert_eq!(
        literal(r#""hi""#),
        Ok(("", ParserLiteral::Str("hi".to_string())))
    );
    assert_eq!(literal("true"), Ok(("", ParserLiteral::Bool(true))));
}

// --- Identifiers ---

#[test]
fn test_identifier_simple() {
    assert_eq!(identifier("x"), Ok(("", "x")));
    assert_eq!(identifier("count"), Ok(("", "count")));
    assert_eq!(identifier("_temp"), Ok(("", "_temp")));
}

#[test]
fn test_identifier_with_digits() {
    assert_eq!(identifier("var1"), Ok(("", "var1")));
    assert_eq!(identifier("max_value_42"), Ok(("", "max_value_42")));
}

#[test]
fn test_identifier_reject_digit_start() {
    assert!(identifier("1var").is_err());
    assert!(identifier("42").is_err());
}

#[test]
fn test_identifier_reject_reserved() {
    assert!(identifier("true").is_err());
    assert!(identifier("false").is_err());
    assert!(identifier("int").is_err());
    assert!(identifier("void").is_err());
}

#[test]
fn test_identifier_accept_true_prefix() {
    assert_eq!(identifier("tru"), Ok(("", "tru")));
    assert_eq!(identifier("truex"), Ok(("", "truex")));
}

// --- Expressions ---

#[test]
fn test_primary_literal() {
    assert_eq!(
        expression("42").map(|(r, e)| (r, e.exp)),
        Ok(("", Expr::Literal(Literal::Int(42))))
    );
    assert_eq!(
        expression("true").map(|(r, e)| (r, e.exp)),
        Ok(("", Expr::Literal(Literal::Bool(true))))
    );
    assert_eq!(
        expression("x").map(|(r, e)| (r, e.exp)),
        Ok(("", Expr::Ident("x".to_string())))
    );
}

#[test]
fn test_arithmetic() {
    assert_eq!(
        expression("1 + 2").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Add(
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(1)),
                    ty: (),
                }),
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(2)),
                    ty: (),
                })
            )
        ))
    );
    assert_eq!(
        expression("10 - 3").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Sub(
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(10)),
                    ty: (),
                }),
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(3)),
                    ty: (),
                })
            )
        ))
    );
    assert_eq!(
        expression("4 * 5").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Mul(
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(4)),
                    ty: (),
                }),
                Box::new(ExprD {
                    exp: Expr::Literal(Literal::Int(5)),
                    ty: (),
                })
            )
        ))
    );
    assert_eq!(
        expression("-x").map(|(r, e)| (r, e.exp)),
        Ok((
            "",
            Expr::Neg(Box::new(ExprD {
                exp: Expr::Ident("x".to_string()),
                ty: (),
            }))
        ))
    );
}

#[test]
fn test_precedence_arithmetic() {
    let result = expression("1 + 2 * 3").unwrap().1.exp;
    match &result {
        Expr::Add(l, r) => {
            assert_eq!(l.exp, Expr::Literal(Literal::Int(1)));
            match &r.exp {
                Expr::Mul(m, n) => {
                    assert_eq!(m.exp, Expr::Literal(Literal::Int(2)));
                    assert_eq!(n.exp, Expr::Literal(Literal::Int(3)));
                }
                _ => panic!("expected Mul"),
            }
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn test_parentheses() {
    let result = expression("(1 + 2) * 3").unwrap().1.exp;
    match &result {
        Expr::Mul(l, r) => {
            match &l.exp {
                Expr::Add(a, b) => {
                    assert_eq!(a.exp, Expr::Literal(Literal::Int(1)));
                    assert_eq!(b.exp, Expr::Literal(Literal::Int(2)));
                }
                _ => panic!("expected Add"),
            }
            assert_eq!(r.exp, Expr::Literal(Literal::Int(3)));
        }
        _ => panic!("expected Mul"),
    }
}

#[test]
fn test_relational() {
    assert!(matches!(expression("a == b").unwrap().1.exp, Expr::Eq(_, _)));
    assert!(matches!(expression("x < 5").unwrap().1.exp, Expr::Lt(_, _)));
    assert!(matches!(expression("1 + 2 < 5").unwrap().1.exp, Expr::Lt(_, _)));
}

#[test]
fn test_complex_expression() {
    // a >= (pi * r * r) + epsilon — area comparison with tolerance
    let result = expression("a >= (pi * r * r) + epsilon").unwrap().1.exp;
    match &result {
        Expr::Ge(left, right) => {
            assert_eq!(left.exp, Expr::Ident("a".to_string()));
            match &right.exp {
                Expr::Add(add_left, add_right) => {
                    assert_eq!(add_right.exp, Expr::Ident("epsilon".to_string()));
                    match &add_left.exp {
                        Expr::Mul(_, _) => {} // (pi * r * r) — parenthesized multiplication
                        _ => panic!("expected Mul for (pi * r * r)"),
                    }
                }
                _ => panic!("expected Add for (pi * r * r) + epsilon"),
            }
        }
        _ => panic!("expected Ge, got {:?}", result),
    }
}

#[test]
fn test_boolean_expr() {
    assert!(matches!(
        expression("true and false").unwrap().1.exp,
        Expr::And(_, _)
    ));
    assert!(matches!(expression("!x").unwrap().1.exp, Expr::Not(_)));
    assert!(matches!(
        expression("x < 5 and y > 0").unwrap().1.exp,
        Expr::And(_, _)
    ));
}

#[test]
fn test_invalid_trailing_op() {
    assert!(all_consuming(expression)("1 +").is_err());
}

#[test]
fn test_invalid_unbalanced_paren() {
    assert!(expression("(1 + 2").is_err());
    assert!(all_consuming(expression)("1 + 2)").is_err());
}

// --- Statements ---

#[test]
fn test_simple_assignment() {
    let result = assignment("x = 42;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Ident(ref s) if s == "x") && value.exp == Expr::Literal(Literal::Int(42))));

    let result = assignment("count = 0;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Ident(ref s) if s == "count") && value.exp == Expr::Literal(Literal::Int(0))));
}

#[test]
fn test_assignment_with_expression() {
    let result = assignment("sum = a + b;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, .. } if matches!(target.exp, Expr::Ident(ref s) if s == "sum")));
    if let Statement::Assign { value, .. } = &result.stmt {
        assert!(matches!(value.exp, Expr::Add(_, _)));
    }

    let result = assignment("flag = x < 5;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, .. } if matches!(target.exp, Expr::Ident(ref s) if s == "flag")));
    if let Statement::Assign { value, .. } = &result.stmt {
        assert!(matches!(value.exp, Expr::Lt(_, _)));
    }
}

#[test]
fn test_assignment_whitespace() {
    let result = assignment("x=1;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Ident(ref s) if s == "x") && value.exp == Expr::Literal(Literal::Int(1))));

    let result = assignment("x = 1;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Ident(ref s) if s == "x") && value.exp == Expr::Literal(Literal::Int(1))));

    let result = assignment("x  =  1;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Ident(ref s) if s == "x") && value.exp == Expr::Literal(Literal::Int(1))));
}

#[test]
fn test_invalid_assignment() {
    assert!(assignment("= 1").is_err());
    assert!(assignment("x").is_err());
    assert!(assignment("1 = x").is_err());
}

#[test]
fn test_decl_statement() {
    let result = statement("int x = 42;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Decl { ref name, ref ty, .. }
        if name == "x" && ty == &Type::Int));
    if let Statement::Decl { ref init, .. } = result.stmt {
        assert_eq!(init.exp, Expr::Literal(Literal::Int(42)));
    }

    let result = statement("float y = 3.14;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Decl { ref name, ref ty, .. }
        if name == "y" && ty == &Type::Float));

    let result = statement("int[] arr = [1, 2, 3];").unwrap().1;
    assert!(matches!(result.stmt, Statement::Decl { ref name, ref ty, .. }
        if name == "arr" && matches!(ty, Type::Array(_))));
}

#[test]
fn test_if_without_else() {
    let result = statement("if x { y = 1; }").unwrap().1;
    assert!(matches!(
        result.stmt,
        Statement::If {
            else_branch: None,
            ..
        }
    ));
    if let Statement::If {
        ref cond, ref then_branch, ..
    } = result.stmt
    {
        assert!(matches!(cond.exp, Expr::Ident(ref s) if s == "x"));
        assert!(matches!(then_branch.stmt, Statement::Block { ref seq }
            if seq.len() == 1
            && matches!(seq[0].stmt, Statement::Assign { ref target, .. }
                if matches!(target.exp, Expr::Ident(ref s) if s == "y"))));
    }
}

#[test]
fn test_if_with_else() {
    let result = statement("if x { y = 1; } else { y = 0; }").unwrap().1;
    assert!(matches!(
        result.stmt,
        Statement::If {
            else_branch: Some(_),
            ..
        }
    ));
    if let Statement::If { ref else_branch, .. } = &result.stmt {
        let else_s = else_branch.as_ref().unwrap();
        assert!(matches!(else_s.stmt, Statement::Block { ref seq }
            if seq.len() == 1
            && matches!(seq[0].stmt, Statement::Assign { ref target, .. }
                if matches!(target.exp, Expr::Ident(ref s) if s == "y"))));
        if let Statement::Block { ref seq } = &else_s.stmt {
            if let Statement::Assign { ref value, .. } = &seq[0].stmt {
                assert_eq!(value.exp, Expr::Literal(Literal::Int(0)));
            }
        }
    }
}

#[test]
fn test_nested_if() {
    let result = statement("if a { if b { x = 1; } else { x = 2; } }").unwrap().1;
    assert!(matches!(result.stmt, Statement::If { .. }));
    if let Statement::If { ref then_branch, .. } = &result.stmt {
        assert!(matches!(then_branch.stmt, Statement::Block { ref seq }
            if seq.len() == 1 && matches!(seq[0].stmt, Statement::If { .. })));
    }
}

#[test]
fn test_if_whitespace() {
    assert!(statement("if x { y = 1; }").is_ok());
    assert!(statement("if  x  { y  =  1; }").is_ok());
}

#[test]
fn test_invalid_if() {
    assert!(statement("if x").is_err());
    assert!(statement("if x y = 1;").is_err());
}

#[test]
fn test_simple_while() {
    let result = statement("while x { y = 1; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
    if let Statement::While { ref cond, ref body } = &result.stmt {
        assert!(matches!(cond.exp, Expr::Ident(ref s) if s == "x"));
        assert!(matches!(body.stmt, Statement::Block { ref seq }
            if seq.len() == 1
            && matches!(seq[0].stmt, Statement::Assign { ref target, .. }
                if matches!(target.exp, Expr::Ident(ref s) if s == "y"))));
    }
}

#[test]
fn test_while_with_expression() {
    let result = statement("while i < 10 { i = i + 1; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
    if let Statement::While { ref cond, ref body } = &result.stmt {
        assert!(matches!(cond.exp, Expr::Lt(_, _)));
        assert!(matches!(body.stmt, Statement::Block { ref seq }
            if seq.len() == 1 && matches!(seq[0].stmt, Statement::Assign { .. })));
    }
}

#[test]
fn test_nested_while() {
    let result = statement("while a { while b { x = 1; } }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
    if let Statement::While { ref body, .. } = &result.stmt {
        assert!(matches!(body.stmt, Statement::Block { ref seq }
            if seq.len() == 1 && matches!(seq[0].stmt, Statement::While { .. })));
    }
}

#[test]
fn test_while_whitespace() {
    assert!(statement("while x { y = 1; }").is_ok());
    assert!(statement("while  x  { y  =  1; }").is_ok());
}

#[test]
fn test_invalid_while() {
    assert!(statement("while x").is_err());
    assert!(statement("while x y = 1;").is_err());
}

// --- Functions ---

#[test]
fn test_fun_decl_with_params() {
    let result = fun_decl("void foo(int x, int y) { x = x + y; }").unwrap().1;
    assert_eq!(result.name, "foo");
    assert_eq!(
        result.params,
        vec![("x".to_string(), Type::Int), ("y".to_string(), Type::Int)]
    );
    assert!(matches!(result.body.stmt, Statement::Block { ref seq }
        if seq.len() == 1
        && matches!(seq[0].stmt, Statement::Assign { ref target, .. }
            if matches!(target.exp, Expr::Ident(ref s) if s == "x"))));
    if let Statement::Block { ref seq } = &result.body.stmt {
        if let Statement::Assign { ref value, .. } = &seq[0].stmt {
            assert!(matches!(value.exp, Expr::Add(_, _)));
        }
    }
}

#[test]
fn test_fun_decl_old_syntax_reject() {
    assert!(fun_decl("def foo(int x) void x = 1").is_err());
    assert!(fun_decl("void bar(x) x = 1").is_err()); // untyped param
}

#[test]
fn test_fun_decl_no_params() {
    let result = fun_decl("void bar() { x = 1; }").unwrap().1;
    assert_eq!(result.name, "bar");
    assert!(result.params.is_empty());
    assert!(
        matches!(result.body.stmt, Statement::Block { ref seq }
            if seq.len() == 1
            && matches!(seq[0].stmt, Statement::Assign { ref target, ref value }
                if matches!(target.exp, Expr::Ident(ref s) if s == "x")
                && value.exp == Expr::Literal(Literal::Int(1))))
    );
}

#[test]
fn test_call_as_expression() {
    let result = expression("foo(1, 2)").unwrap().1;
    assert!(
        matches!(result.exp, Expr::Call { ref name, ref args } if name == "foo" && args.len() == 2)
    );
    if let Expr::Call { ref args, .. } = result.exp {
        assert_eq!(args[0].exp, Expr::Literal(Literal::Int(1)));
        assert_eq!(args[1].exp, Expr::Literal(Literal::Int(2)));
    }
}

#[test]
fn test_call_no_args() {
    let result = expression("baz()").unwrap().1;
    assert!(
        matches!(result.exp, Expr::Call { ref name, ref args } if name == "baz" && args.is_empty())
    );
}

#[test]
fn test_call_in_expression() {
    let result = expression("foo(1) + 2").unwrap().1;
    assert!(matches!(result.exp, Expr::Add(_, _)));
    if let Expr::Add(ref left, ref right) = result.exp {
        assert!(
            matches!(left.exp, Expr::Call { ref name, ref args } if name == "foo" && args.len() == 1)
        );
        assert_eq!(right.exp, Expr::Literal(Literal::Int(2)));
    }
}

#[test]
fn test_call_as_statement() {
    let result = statement("foo(1, 2);").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Call { ref name, ref args } if name == "foo" && args.len() == 2)
    );
}

// --- Blocks ---

#[test]
fn test_empty_block() {
    let result = statement("{}").unwrap().1;
    assert!(matches!(result.stmt, Statement::Block { ref seq } if seq.is_empty()));
}

#[test]
fn test_block_single_statement() {
    let result = statement("{ x = 1; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::Block { ref seq } if seq.len() == 1));
    if let Statement::Block { ref seq } = result.stmt {
        assert!(matches!(seq[0].stmt, Statement::Assign { ref target, .. } if matches!(target.exp, Expr::Ident(ref s) if s == "x")));
    }
}

#[test]
fn test_block_multiple_statements() {
    let result = statement("{ x = 1; y = 2; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::Block { ref seq } if seq.len() == 2));
    if let Statement::Block { ref seq } = result.stmt {
        assert!(matches!(seq[0].stmt, Statement::Assign { ref target, .. } if matches!(target.exp, Expr::Ident(ref s) if s == "x")));
        assert!(matches!(seq[1].stmt, Statement::Assign { ref target, .. } if matches!(target.exp, Expr::Ident(ref s) if s == "y")));
    }
}

#[test]
fn test_block_in_function_body() {
    let result = fun_decl("void foo(int x, int y) { x = x + 1; y = y + 1; }")
        .unwrap()
        .1;
    assert!(matches!(result.body.stmt, Statement::Block { ref seq } if seq.len() == 2));
}

#[test]
fn test_block_in_if_body() {
    let result = statement("if x { a = 1; b = 2; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::If { .. }));
    if let Statement::If { ref then_branch, .. } = &result.stmt {
        assert!(matches!(then_branch.stmt, Statement::Block { ref seq } if seq.len() == 2));
    }
}

#[test]
fn test_block_in_while_body() {
    let result = statement("while x { a = 1; b = 2; }").unwrap().1;
    assert!(matches!(result.stmt, Statement::While { .. }));
    if let Statement::While { ref body, .. } = &result.stmt {
        assert!(matches!(body.stmt, Statement::Block { ref seq } if seq.len() == 2));
    }
}

// --- Arrays ---

#[test]
fn test_array_literal() {
    let result = expression("[1, 2, 3]").unwrap().1;
    assert!(matches!(result.exp, Expr::ArrayLit(ref elems) if elems.len() == 3));
    if let Expr::ArrayLit(ref elems) = result.exp {
        assert_eq!(elems[0].exp, Expr::Literal(Literal::Int(1)));
        assert_eq!(elems[1].exp, Expr::Literal(Literal::Int(2)));
        assert_eq!(elems[2].exp, Expr::Literal(Literal::Int(3)));
    }
}

#[test]
fn test_empty_array() {
    let result = expression("[]").unwrap().1;
    assert!(matches!(result.exp, Expr::ArrayLit(ref elems) if elems.is_empty()));
}

#[test]
fn test_index_read() {
    let result = expression("arr[i]").unwrap().1;
    assert!(matches!(result.exp, Expr::Index { ref base, ref index }
        if matches!(base.exp, Expr::Ident(ref s) if s == "arr") && matches!(index.exp, Expr::Ident(ref s) if s == "i")));
}

#[test]
fn test_indexed_assignment() {
    let result = statement("arr[i] = 1;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Index { .. }) && value.exp == Expr::Literal(Literal::Int(1))));
    if let Statement::Assign { ref target, .. } = result.stmt {
        if let Expr::Index { ref base, ref index } = target.exp {
            assert!(matches!(base.exp, Expr::Ident(ref s) if s == "arr"));
            assert!(matches!(index.exp, Expr::Ident(ref s) if s == "i"));
        }
    }
}

#[test]
fn test_multidimensional_indexed_assignment() {
    let result = statement("arr[i][j] = x;").unwrap().1;
    assert!(matches!(result.stmt, Statement::Assign { ref target, ref value }
        if matches!(target.exp, Expr::Index { .. }) && matches!(value.exp, Expr::Ident(ref s) if s == "x")));
    if let Statement::Assign { ref target, .. } = result.stmt {
        if let Expr::Index { ref base, ref index } = target.exp {
            assert!(matches!(index.exp, Expr::Ident(ref s) if s == "j"));
            if let Expr::Index { ref base, ref index } = base.exp {
                assert!(matches!(base.exp, Expr::Ident(ref s) if s == "arr"));
                assert!(matches!(index.exp, Expr::Ident(ref s) if s == "i"));
            }
        }
    }
}

#[test]
fn test_nested_index() {
    let result = expression("arr[i][j]").unwrap().1;
    assert!(matches!(result.exp, Expr::Index { ref base, ref index }
        if matches!(index.exp, Expr::Ident(ref s) if s == "j")));
    if let Expr::Index { ref base, .. } = result.exp {
        assert!(matches!(base.exp, Expr::Index { ref base, ref index }
            if matches!(base.exp, Expr::Ident(ref s) if s == "arr") && matches!(index.exp, Expr::Ident(ref s) if s == "i")));
    }
}

#[test]
fn test_array_in_expression() {
    let result = expression("[1, 2][0]").unwrap().1;
    assert!(matches!(result.exp, Expr::Index { ref base, ref index }
        if matches!(base.exp, Expr::ArrayLit(_)) && index.exp == Expr::Literal(Literal::Int(0))));
}

// --- Pointers (`Type::Pointer` + `type_name`: int*, float*, …) ---

#[test]
fn test_pointer_type_name() {
    assert_eq!(
        type_name("int*"),
        Ok(("", Type::Pointer(Box::new(Type::Int))))
    );
    assert_eq!(
        type_name("float*"),
        Ok(("", Type::Pointer(Box::new(Type::Float))))
    );
    assert_eq!(
        type_name("bool*"),
        Ok(("", Type::Pointer(Box::new(Type::Bool))))
    );
    assert_eq!(
        type_name("str*"),
        Ok(("", Type::Pointer(Box::new(Type::Str))))
    );
    // `int*` deve ser reconhecido antes de `int` (caso contrário vira só Int).
    assert_eq!(type_name("int"), Ok(("", Type::Int)));
}

#[test]
fn test_pointer_variable_declaration() {
    let result = statement("int* ptr = q;").unwrap().1;
    assert!(matches!(
        result.stmt,
        Statement::Decl {
            ref name,
            ref ty,
            ..
        } if name == "ptr" && ty == &Type::Pointer(Box::new(Type::Int))
    ));
    if let Statement::Decl { ref init, .. } = result.stmt {
        assert!(matches!(init.exp, Expr::Ident(ref s) if s == "q"));
    }
}

#[test]
fn test_pointer_address_of() {
    let result = expression("&x").unwrap().1;
    assert!(matches!(result.exp, Expr::AddrOf(ref target) if matches!(target.exp, Expr::Ident(ref s) if s == "x")));
}

#[test]
fn test_pointer_dereference() {
    let result = expression("*p").unwrap().1;
    assert!(matches!(result.exp, Expr::Deref(ref target) if matches!(target.exp, Expr::Ident(ref s) if s == "p")));
}

#[test]
fn test_pointer_params_function() {
    let result = fun_decl("void foo(int* p) { *p = 42; }").unwrap().1;
    assert_eq!(result.name, "foo");
    assert_eq!(
        result.params,
        vec![("p".to_string(), Type::Pointer(Box::new(Type::Int)))]
    );
    assert!(matches!(result.body.stmt, Statement::Block { ref seq } if seq.len() == 1));
    if let Statement::Block { ref seq } = &result.body.stmt {
        assert!(matches!(seq[0].stmt, Statement::Assign { ref target, ref value }
            if matches!(target.exp, Expr::Deref(ref t) if matches!(t.exp, Expr::Ident(ref s) if s == "p"))
            && value.exp == Expr::Literal(Literal::Int(42))));
    }
}

#[test]
fn test_pointer_type_function() {
    let result = fun_decl("int* changeRef(int* x, int* y) { x = y; return x; }")
        .unwrap()
        .1;
    assert_eq!(result.name, "changeRef");
    assert_eq!(
        result.params,
        vec![
            ("x".to_string(), Type::Pointer(Box::new(Type::Int))),
            ("y".to_string(), Type::Pointer(Box::new(Type::Int)))
        ]
    );
}