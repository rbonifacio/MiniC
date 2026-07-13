use mini_c::ir::ast::{Expr, IdentifierDecl, Statement, Type};
use mini_c::parser::{expression, fun_decl, statement};

#[test]
fn fun_decl_with_params() {
    let result = fun_decl("void foo(int x, int y) { x = x + y; }").unwrap().1;
    assert_eq!(result.name, "foo");
    assert_eq!(
        result.params,
        vec![
            IdentifierDecl {
                name: "x".to_string(),
                ty: Type::Int
            },
            IdentifierDecl {
                name: "y".to_string(),
                ty: Type::Int
            },
        ]
    );
}

#[test]
fn fun_decl_old_syntax_reject() {
    assert!(fun_decl("def foo(int x) void x = 1").is_err());
    assert!(fun_decl("void bar(x) x = 1").is_err());
}

#[test]
fn fun_decl_no_params() {
    let result = fun_decl("void bar() { x = 1; }").unwrap().1;
    assert_eq!(result.name, "bar");
    assert!(result.params.is_empty());
}

#[test]
fn call_as_expression() {
    let result = expression("foo(1, 2)").unwrap().1;
    assert!(
        matches!(result.exp, Expr::Call { ref name, ref args } if name == "foo" && args.len() == 2)
    );
}

#[test]
fn call_no_args() {
    let result = expression("baz()").unwrap().1;
    assert!(
        matches!(result.exp, Expr::Call { ref name, ref args } if name == "baz" && args.is_empty())
    );
}

#[test]
fn call_in_expression() {
    let result = expression("foo(1) + 2").unwrap().1;
    assert!(matches!(result.exp, Expr::Add(_, _)));
}

#[test]
fn call_as_statement() {
    let result = statement("foo(1, 2);").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Call { ref name, ref args } if name == "foo" && args.len() == 2)
    );
}
