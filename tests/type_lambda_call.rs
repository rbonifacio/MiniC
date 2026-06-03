use mini_c::{parser::program, semantic::type_check};

fn type_check_src(src: &str) -> Result<(), String> {
    let (_, unchecked) = program(src).map_err(|e| format!("parse error: {:?}", e))?;
    type_check(&unchecked).map(|_| ()).map_err(|e| format!("type error: {}", e.message))
}

#[test]
fn test_lambda_type_check_ok() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int { return x * 2; };
            int r = f(3);
        }
    "#;
    assert!(type_check_src(src).is_ok(), "lambda should type-check");
}

#[test]
fn test_lambda_call_wrong_arg() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int { return x; };
            f(true);
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "expected type error when calling with wrong arg type");
}
