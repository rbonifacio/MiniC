//! Integration tests for the MiniC parser.

use MiniC::ir::ast::{Expr, Literal, Stmt};
use MiniC::parser::{
    assignment, expression, fun_decl, identifier, literal, statement,
    literals::{integer_literal, float_literal, string_literal, boolean_literal, Literal as ParserLiteral},
};
use nom::combinator::all_consuming;

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
    assert_eq!(string_literal(r#""line1\nline2""#), Ok(("", "line1\nline2".to_string())));
    assert_eq!(string_literal(r#""tab\there""#), Ok(("", "tab\there".to_string())));
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
    assert_eq!(literal(r#""hi""#), Ok(("", ParserLiteral::Str("hi".to_string()))));
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
        expression("42"),
        Ok(("", Expr::Literal(Literal::Int(42))))
    );
    assert_eq!(
        expression("true"),
        Ok(("", Expr::Literal(Literal::Bool(true))))
    );
    assert_eq!(
        expression("x"),
        Ok(("", Expr::Ident("x".to_string())))
    );
}

#[test]
fn test_arithmetic() {
    assert_eq!(
        expression("1 + 2"),
        Ok(("", Expr::Add(Box::new(Expr::Literal(Literal::Int(1))), Box::new(Expr::Literal(Literal::Int(2))))))
    );
    assert_eq!(
        expression("10 - 3"),
        Ok(("", Expr::Sub(Box::new(Expr::Literal(Literal::Int(10))), Box::new(Expr::Literal(Literal::Int(3))))))
    );
    assert_eq!(
        expression("4 * 5"),
        Ok(("", Expr::Mul(Box::new(Expr::Literal(Literal::Int(4))), Box::new(Expr::Literal(Literal::Int(5))))))
    );
    assert_eq!(
        expression("-x"),
        Ok(("", Expr::Neg(Box::new(Expr::Ident("x".to_string())))))
    );
}

#[test]
fn test_precedence_arithmetic() {
    let result = expression("1 + 2 * 3").unwrap().1;
    match &result {
        Expr::Add(l, r) => {
            assert_eq!(*l.clone(), Expr::Literal(Literal::Int(1)));
            match r.as_ref() {
                Expr::Mul(m, n) => {
                    assert_eq!(*m.clone(), Expr::Literal(Literal::Int(2)));
                    assert_eq!(*n.clone(), Expr::Literal(Literal::Int(3)));
                }
                _ => panic!("expected Mul"),
            }
        }
        _ => panic!("expected Add"),
    }
}

#[test]
fn test_parentheses() {
    let result = expression("(1 + 2) * 3").unwrap().1;
    match &result {
        Expr::Mul(l, r) => {
            match l.as_ref() {
                Expr::Add(a, b) => {
                    assert_eq!(*a.clone(), Expr::Literal(Literal::Int(1)));
                    assert_eq!(*b.clone(), Expr::Literal(Literal::Int(2)));
                }
                _ => panic!("expected Add"),
            }
            assert_eq!(*r.clone(), Expr::Literal(Literal::Int(3)));
        }
        _ => panic!("expected Mul"),
    }
}

#[test]
fn test_relational() {
    assert!(matches!(
        expression("a == b").unwrap().1,
        Expr::Eq(_, _)
    ));
    assert!(matches!(
        expression("x < 5").unwrap().1,
        Expr::Lt(_, _)
    ));
    assert!(matches!(
        expression("1 + 2 < 5").unwrap().1,
        Expr::Lt(_, _)
    ));
}

#[test]
fn test_complex_expression() {
    // a >= (pi * r * r) + epsilon — area comparison with tolerance
    let result = expression("a >= (pi * r * r) + epsilon").unwrap().1;
    match &result {
        Expr::Ge(left, right) => {
            assert_eq!(left.as_ref(), &Expr::Ident("a".to_string()));
            match right.as_ref() {
                Expr::Add(add_left, add_right) => {
                    assert_eq!(add_right.as_ref(), &Expr::Ident("epsilon".to_string()));
                    match add_left.as_ref() {
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
        expression("true and false").unwrap().1,
        Expr::And(_, _)
    ));
    assert!(matches!(
        expression("!x").unwrap().1,
        Expr::Not(_)
    ));
    assert!(matches!(
        expression("x < 5 and y > 0").unwrap().1,
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
    let result = assignment("x = 42").unwrap().1;
    assert!(matches!(result, Stmt::Assign { target, value } if target == "x" && *value == Expr::Literal(Literal::Int(42))));

    let result = assignment("count = 0").unwrap().1;
    assert!(matches!(result, Stmt::Assign { target, value } if target == "count" && *value == Expr::Literal(Literal::Int(0))));
}

#[test]
fn test_assignment_with_expression() {
    let result = assignment("sum = a + b").unwrap().1;
    assert!(matches!(result, Stmt::Assign { ref target, .. } if target == "sum"));
    if let Stmt::Assign { value, .. } = result {
        assert!(matches!(value.as_ref(), Expr::Add(_, _)));
    }

    let result = assignment("flag = x < 5").unwrap().1;
    assert!(matches!(result, Stmt::Assign { ref target, .. } if target == "flag"));
    if let Stmt::Assign { value, .. } = result {
        assert!(matches!(value.as_ref(), Expr::Lt(_, _)));
    }
}

#[test]
fn test_assignment_whitespace() {
    let result = assignment("x=1").unwrap().1;
    assert!(matches!(result, Stmt::Assign { target, value } if target == "x" && *value == Expr::Literal(Literal::Int(1))));

    let result = assignment("x = 1").unwrap().1;
    assert!(matches!(result, Stmt::Assign { target, value } if target == "x" && *value == Expr::Literal(Literal::Int(1))));

    let result = assignment("x  =  1").unwrap().1;
    assert!(matches!(result, Stmt::Assign { target, value } if target == "x" && *value == Expr::Literal(Literal::Int(1))));
}

#[test]
fn test_invalid_assignment() {
    assert!(assignment("= 1").is_err());
    assert!(assignment("x").is_err());
    assert!(assignment("1 = x").is_err());
}

#[test]
fn test_if_without_else() {
    let result = statement("if x then y = 1").unwrap().1;
    assert!(matches!(result, Stmt::If { else_branch: None, .. }));
    if let Stmt::If { cond, then_branch, .. } = result {
        assert!(matches!(cond.as_ref(), Expr::Ident(s) if s == "x"));
        assert!(matches!(then_branch.as_ref(), Stmt::Assign { target, .. } if target == "y"));
    }
}

#[test]
fn test_if_with_else() {
    let result = statement("if x then y = 1 else y = 0").unwrap().1;
    assert!(matches!(result, Stmt::If { else_branch: Some(_), .. }));
    if let Stmt::If { else_branch, .. } = result {
        let else_s = else_branch.unwrap();
        assert!(matches!(else_s.as_ref(), Stmt::Assign { ref target, .. } if target == "y"));
        if let Stmt::Assign { value, .. } = else_s.as_ref() {
            assert_eq!(value.as_ref(), &Expr::Literal(Literal::Int(0)));
        }
    }
}

#[test]
fn test_nested_if() {
    let result = statement("if a then if b then x = 1 else x = 2").unwrap().1;
    assert!(matches!(result, Stmt::If { .. }));
    if let Stmt::If { then_branch, .. } = result {
        assert!(matches!(then_branch.as_ref(), Stmt::If { .. }));
    }
}

#[test]
fn test_if_whitespace() {
    assert!(statement("if x then y=1").is_ok());
    assert!(statement("if  x  then  y  =  1").is_ok());
}

#[test]
fn test_invalid_if() {
    assert!(statement("if x").is_err());
    assert!(statement("if then x = 1").is_err());
}

#[test]
fn test_simple_while() {
    let result = statement("while x do y = 1").unwrap().1;
    assert!(matches!(result, Stmt::While { .. }));
    if let Stmt::While { cond, body } = result {
        assert!(matches!(cond.as_ref(), Expr::Ident(s) if s == "x"));
        assert!(matches!(body.as_ref(), Stmt::Assign { target, .. } if target == "y"));
    }
}

#[test]
fn test_while_with_expression() {
    let result = statement("while i < 10 do i = i + 1").unwrap().1;
    assert!(matches!(result, Stmt::While { .. }));
    if let Stmt::While { cond, body } = result {
        assert!(matches!(cond.as_ref(), Expr::Lt(_, _)));
        assert!(matches!(body.as_ref(), Stmt::Assign { .. }));
    }
}

#[test]
fn test_nested_while() {
    let result = statement("while a do while b do x = 1").unwrap().1;
    assert!(matches!(result, Stmt::While { .. }));
    if let Stmt::While { body, .. } = result {
        assert!(matches!(body.as_ref(), Stmt::While { .. }));
    }
}

#[test]
fn test_while_whitespace() {
    assert!(statement("while x do y=1").is_ok());
    assert!(statement("while  x  do  y  =  1").is_ok());
}

#[test]
fn test_invalid_while() {
    assert!(statement("while x").is_err());
    assert!(statement("while do x = 1").is_err());
}

// --- Functions ---

#[test]
fn test_fun_decl_with_params() {
    let result = fun_decl("def foo(x, y) x = x + y").unwrap().1;
    assert_eq!(result.name, "foo");
    assert_eq!(result.params, vec!["x", "y"]);
    assert!(matches!(result.body.as_ref(), Stmt::Assign { target, .. } if target == "x"));
    if let Stmt::Assign { value, .. } = result.body.as_ref() {
        assert!(matches!(value.as_ref(), Expr::Add(_, _)));
    }
}

#[test]
fn test_fun_decl_no_params() {
    let result = fun_decl("def bar() x = 1").unwrap().1;
    assert_eq!(result.name, "bar");
    assert!(result.params.is_empty());
    assert!(matches!(result.body.as_ref(), Stmt::Assign { target, value } if target == "x" && value.as_ref() == &Expr::Literal(Literal::Int(1))));
}

#[test]
fn test_call_as_expression() {
    let result = expression("foo(1, 2)").unwrap().1;
    assert!(matches!(result, Expr::Call { ref name, ref args } if name == "foo" && args.len() == 2));
    if let Expr::Call { args, .. } = result {
        assert_eq!(args[0], Expr::Literal(Literal::Int(1)));
        assert_eq!(args[1], Expr::Literal(Literal::Int(2)));
    }
}

#[test]
fn test_call_no_args() {
    let result = expression("baz()").unwrap().1;
    assert!(matches!(result, Expr::Call { ref name, ref args } if name == "baz" && args.is_empty()));
}

#[test]
fn test_call_in_expression() {
    let result = expression("foo(1) + 2").unwrap().1;
    assert!(matches!(result, Expr::Add(_, _)));
    if let Expr::Add(left, right) = result {
        assert!(matches!(left.as_ref(), Expr::Call { ref name, ref args } if name == "foo" && args.len() == 1));
        assert_eq!(*right, Expr::Literal(Literal::Int(2)));
    }
}

#[test]
fn test_call_as_statement() {
    let result = statement("foo(1, 2)").unwrap().1;
    assert!(matches!(result, Stmt::Call { ref name, ref args } if name == "foo" && args.len() == 2));
}

// --- Blocks ---

#[test]
fn test_empty_block() {
    let result = statement("{}").unwrap().1;
    assert!(matches!(result, Stmt::Block { ref seq } if seq.is_empty()));
}

#[test]
fn test_block_single_statement() {
    let result = statement("{ x = 1 }").unwrap().1;
    assert!(matches!(result, Stmt::Block { ref seq } if seq.len() == 1));
    if let Stmt::Block { seq } = result {
        assert!(matches!(seq[0], Stmt::Assign { ref target, .. } if target == "x"));
    }
}

#[test]
fn test_block_multiple_statements() {
    let result = statement("{ x = 1; y = 2 }").unwrap().1;
    assert!(matches!(result, Stmt::Block { ref seq } if seq.len() == 2));
    if let Stmt::Block { seq } = result {
        assert!(matches!(seq[0], Stmt::Assign { ref target, .. } if target == "x"));
        assert!(matches!(seq[1], Stmt::Assign { ref target, .. } if target == "y"));
    }
}

#[test]
fn test_block_in_function_body() {
    let result = fun_decl("def foo(x, y) { x = x + 1; y = y + 1 }").unwrap().1;
    assert!(matches!(result.body.as_ref(), Stmt::Block { ref seq } if seq.len() == 2));
}

#[test]
fn test_block_in_if_body() {
    let result = statement("if x then { a = 1; b = 2 }").unwrap().1;
    assert!(matches!(result, Stmt::If { .. }));
    if let Stmt::If { then_branch, .. } = result {
        assert!(matches!(then_branch.as_ref(), Stmt::Block { ref seq } if seq.len() == 2));
    }
}

#[test]
fn test_block_in_while_body() {
    let result = statement("while x do { a = 1; b = 2 }").unwrap().1;
    assert!(matches!(result, Stmt::While { .. }));
    if let Stmt::While { body, .. } = result {
        assert!(matches!(body.as_ref(), Stmt::Block { ref seq } if seq.len() == 2));
    }
}
