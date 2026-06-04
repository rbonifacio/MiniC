use mini_c::interpreter::interpret;
use mini_c::parser::program;
use mini_c::semantic::type_check;

fn run(src: &str) -> Result<(), String> {
    let (_, unchecked) = program(src).map_err(|e| format!("parse error: {:?}", e))?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
}

#[test]
fn test_fn_value_runtime() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int { return x * 2; };
            print(f(21));
        }
    "#;
    assert!(run(src).is_ok(), "expected runtime to succeed");
}

#[test]
fn test_fn_value_call_through_variable() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int { return x * 2; };
            fn(int) -> int g = f;
            print(g(21));
        }
    "#;
    assert!(run(src).is_ok(), "expected function call through variable to succeed");
}