use mini_c::{parser::program, semantic::type_check};

fn type_check_src(src: &str) -> Result<(), String> {
    let (_, unchecked) = program(src).map_err(|e| format!("parse error: {:?}", e))?;
    type_check(&unchecked).map(|_| ()).map_err(|e| format!("type error: {}", e.message))
}

#[test]
fn test_assign_compatible_fn_type() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int { return x; };
            int r = f(5);
        }
    "#;
    assert!(type_check_src(src).is_ok(), "expected type-check to succeed");
}

#[test]
fn test_assign_incompatible_fn_type() {
    let src = r#"
        void main() {
            fn(float) -> int f = fn(int x) -> int { return x; };
        }
    "#;
    let res = type_check_src(src);
    assert!(res.is_err(), "expected type error for incompatible function assignment");
}
