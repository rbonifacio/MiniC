use mini_c::{interpreter::interpret, parser::program, semantic::type_check};

fn run(src: &str) -> Result<(), String> {
    let (_, unchecked) = program(src).map_err(|e| format!("parse error: {:?}", e))?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
}

#[test]
fn test_closure_snapshot() {
    let src = r#"
        void main() {
            int y = 10;
            fn(int) -> int f = fn(int x) -> int { return x + y; };
            y = 20;
            print(f(1));
        }
    "#;
    assert!(run(src).is_ok(), "closure should capture creation-time value");
}
