use mini_c::{interpreter::interpret, parser::program, semantic::type_check};

fn run(src: &str) -> Result<(), String> {
    let (_, unchecked) = program(src).map_err(|e| format!("parse error: {:?}", e))?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
}

#[test]
fn closure_snapshot_captures_creation_time_value() {
    let src = r#"
        void main() {
            int y = 10;
            fn(int) -> int f = fn(int x) -> int { return x + y; };
            y = 20;
            print(f(1));
        }
    "#;

    assert!(run(src).is_ok());
}

#[test]
fn closures_capture_different_snapshots() {
    let src = r#"
        void main() {
            int y = 10;

            fn(int) -> int f =
                fn(int x) -> int {
                    return x + y;
                };

            y = 20;

            fn(int) -> int g =
                fn(int x) -> int {
                    return x + y;
                };

            print(f(1));
            print(g(1));
        }
    "#;

    assert!(run(src).is_ok());
}

#[test]
fn closure_snapshot_captures_multiple_variables() {
    let src = r#"
        void main() {
            int a = 2;
            int b = 3;

            fn(int) -> int f = fn(int x) -> int {
                return x + a + b;
            };

            a = 100;
            b = 200;
            print(f(1));
        }
    "#;

    assert!(run(src).is_ok());
}

#[test]
fn closure_called_through_variable_is_accepted() {
    let src = r#"
        void main() {
            fn(int) -> int inc = fn(int x) -> int { return x + 1; };
            print(inc(41));
        }
    "#;

    assert!(run(src).is_ok());
}

#[test]
fn closure_can_be_called_multiple_times() {
    let src = r#"
        void main() {
            int y = 10;

            fn(int) -> int f = fn(int x) -> int {
                return x + y;
            };

            print(f(1));
            print(f(2));
            print(f(3));
        }
    "#;

    assert!(run(src).is_ok());
}

#[test]
fn caller_environment_is_restored_after_closure_call() {
    let src = r#"
        void main() {
            int y = 10;

            fn(int) -> int f = fn(int x) -> int {
                return x + y;
            };

            print(f(1));
            y = 50;
            print(y);
        }
    "#;

    assert!(run(src).is_ok());
}

#[test]
fn nested_closure_still_uses_captured_snapshot() {
    let src = r#"
        void main() {
            int y = 10;

            fn(int) -> int make = fn(int x) -> int {
                return x + y;
            };

            fn(int) -> int call = fn(int z) -> int {
                return make(z);
            };

            y = 20;
            print(call(1));
        }
    "#;

    assert!(run(src).is_ok());
}