use mini_c::{parser::program, semantic::type_check};

fn type_check_src(src: &str) -> Result<(), String> {
    let (rest, unchecked) = program(src)
        .map_err(|e| format!("parse error: {:?}", e))?;

    if !rest.trim().is_empty() {
        return Err(format!("unparsed input: {:?}", rest));
    }

    type_check(&unchecked)
        .map(|_| ())
        .map_err(|e| format!("type error: {}", e.message))
}

#[test]
fn test_lambda_type_check_ok() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int {
                return x * 2;
            };

            int r = f(3);
        }
    "#;

    assert!(
        type_check_src(src).is_ok(),
        "lambda should type-check"
    );
}

#[test]
fn test_lambda_call_wrong_arg() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int {
                return x;
            };

            f(true);
        }
    "#;

    let res = type_check_src(src);

    assert!(
        res.is_err(),
        "expected type error when calling with wrong arg type"
    );
}

#[test]
fn test_lambda_wrong_return_type() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int {
                return true;
            };
        }
    "#;

    let res = type_check_src(src);

    assert!(
        res.is_err(),
        "expected type error when lambda returns bool instead of int"
    );
}

#[test]
fn test_lambda_call_wrong_number_of_args() {
    let src = r#"
        void main() {
            fn(int) -> int f = fn(int x) -> int {
                return x;
            };

            f();
        }
    "#;

    let res = type_check_src(src);

    assert!(
        res.is_err(),
        "expected type error when calling function with wrong number of args"
    );
}

#[test]
fn test_call_non_function_value() {
    let src = r#"
        void main() {
            int x = 10;
            x(1);
        }
    "#;

    let res = type_check_src(src);

    assert!(
        res.is_err(),
        "expected type error when calling a non-function value"
    );
}