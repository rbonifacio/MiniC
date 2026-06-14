use mini_c::{
    environment::Environment,
    interpreter::exec_stmt::exec_stmt,
    interpreter::interpret,
    interpreter::value::{FnValue, Value},
    ir::ast::Statement,
    parser::program,
    semantic::type_check,
    stdlib::NativeRegistry,
};

/// Parse, type-check, and interpret a MiniC source string.
fn run(src: &str) -> Result<(), String> {
    let unchecked = program(src)
        .map_err(|e| format!("parse error: {:?}", e))
        .map(|(_, p)| p)?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
}

/// Parse, type-check, then execute `main`'s statements directly in a fresh
/// environment and hand that environment back so individual tests can inspect
/// the final value of a variable. Unlike [`interpret`], which discards all
/// state, this lets us assert on what a loop actually computed.
fn run_main_and_inspect(src: &str) -> Environment<Value> {
    let unchecked = program(src).expect("parse should succeed").1;
    let checked = type_check(&unchecked).expect("type check should succeed");

    // Register stdlib and user functions exactly as `interpret` does, so the
    // executed statements can call functions and built-ins like `print`.
    let mut env = Environment::<Value>::new();
    let registry = NativeRegistry::default();
    for (name, entry) in registry.iter() {
        env.declare(name.clone(), Value::Fn(FnValue::Native(entry.func)));
    }
    for fun in &checked.functions {
        env.declare(fun.name.clone(), Value::Fn(FnValue::UserDefined(fun.clone())));
    }

    let main_fn = checked
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("main must exist");

    // Run the body's statements one by one (rather than as a single block) so
    // that `main`'s own locals are not cleaned up before we can read them.
    match &main_fn.body.stmt {
        Statement::Block { seq } => {
            for s in seq {
                exec_stmt(s, &mut env).expect("execution should succeed");
            }
        }
        _ => panic!("main body must be a block"),
    }
    env
}

// ---------------------------------------------------------------------------
// 7.2 Empty main
// ---------------------------------------------------------------------------
#[test]
fn test_empty_main() {
    let src = "void main() {}";
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.3 Arithmetic
// ---------------------------------------------------------------------------
#[test]
fn test_arithmetic_int() {
    let src = r#"
        int add(int a, int b) { return a + b; }
        void main() { int r = add(3, 4); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

#[test]
fn test_arithmetic_float_coercion() {
    let src = r#"
        float f() { return 2 + 1.5; }
        void main() { float r = f(); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.4 If/else
// ---------------------------------------------------------------------------
#[test]
fn test_if_true_branch() {
    let src = r#"
        int choose(bool cond) {
            if cond { return 1; } else { return 2; }
            return 0;
        }
        void main() { int r = choose(true); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

#[test]
fn test_if_false_branch() {
    let src = r#"
        int choose(bool cond) {
            if cond { return 1; } else { return 2; }
            return 0;
        }
        void main() { int r = choose(false); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.5 While loop
// ---------------------------------------------------------------------------
#[test]
fn test_while_loop() {
    let src = r#"
        int count_to(int n) {
            int i = 0;
            while i < n { i = i + 1; }
            return i;
        }
        void main() { int r = count_to(3); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

#[test]
fn test_while_no_iteration() {
    let src = r#"
        void main() {
            int x = 0;
            while false { x = 1; }
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.6 Recursion
// ---------------------------------------------------------------------------
#[test]
fn test_factorial() {
    let src = r#"
        int factorial(int n) {
            if n <= 1 { return 1; }
            return n * factorial(n - 1);
        }
        void main() { int r = factorial(5); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.7 Array declaration, assignment, and read
// ---------------------------------------------------------------------------
#[test]
fn test_array_decl_and_index() {
    let src = r#"
        void main() {
            int[] arr = [10, 20, 30];
            int x = arr[1];
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

#[test]
fn test_array_element_assignment() {
    let src = r#"
        void main() {
            int[] arr = [1, 2, 3];
            arr[0] = 99;
            int x = arr[0];
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.8 Nested array element assignment
// ---------------------------------------------------------------------------
#[test]
fn test_nested_array_assignment() {
    let src = r#"
        void main() {
            int[] row0 = [1, 2];
            int[] row1 = [3, 4];
            int[][] matrix = [row0, row1];
            matrix[1][0] = 99;
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.9 print
// ---------------------------------------------------------------------------
#[test]
fn test_print_int() {
    let src = r#"
        void main() { print(42); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

#[test]
fn test_print_bool() {
    let src = r#"
        void main() { print(true); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

#[test]
fn test_print_array() {
    let src = r#"
        void main() { print([1, 2, 3]); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.10 Out-of-bounds array access
// ---------------------------------------------------------------------------
#[test]
fn test_out_of_bounds() {
    let src = r#"
        void main() {
            int[] arr = [1, 2];
            int x = arr[5];
        }
    "#;
    let result = run(src);
    assert!(result.is_err(), "expected out-of-bounds error");
    assert!(
        result.unwrap_err().contains("out of bounds"),
        "error should mention 'out of bounds'"
    );
}

// ---------------------------------------------------------------------------
// 7.11 Undefined function (caught by type checker)
// ---------------------------------------------------------------------------
#[test]
fn test_undefined_function() {
    let src = r#"
        void main() { foo(1); }
    "#;
    assert!(run(src).is_err(), "expected error for undefined function");
}

// ---------------------------------------------------------------------------
// 7.4 sqrt via native registry
// ---------------------------------------------------------------------------
#[test]
fn test_stdlib_sqrt_int_coercion() {
    let src = r#"
        void main() { float r = sqrt(4); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.5 pow via native registry
// ---------------------------------------------------------------------------
#[test]
fn test_stdlib_pow_int_args() {
    let src = r#"
        void main() { float r = pow(2, 10); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.4 readInt, readFloat, readString are registered (type-check passes)
// ---------------------------------------------------------------------------
#[test]
fn test_stdlib_read_fns_type_check() {
    // These functions are registered; the program should type-check even if
    // we don't call them at runtime (call sites are inside dead branches).
    let src = r#"
        void main() {
            if false { int x = readInt(); }
            if false { float x = readFloat(); }
            if false { str x = readString(); }
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// 7.5b pow(2.0, 3.0) returns 8.0 via unified dispatch
// ---------------------------------------------------------------------------
#[test]
fn test_stdlib_pow_float_args() {
    let src = r#"
        void main() { float r = pow(2.0, 3.0); }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// ---------------------------------------------------------------------------
// For statement (Milestone 2): execution semantics.
// ---------------------------------------------------------------------------

// The canonical example: summing 0..9 yields 45, and the loop runs end-to-end
// through the full pipeline (including `print`).
#[test]
fn test_for_full_pipeline_with_print() {
    let src = r#"
        void main() {
            int sum = 0;
            for (int i = 0; i < 10; i = i + 1) { sum = sum + i; }
            print(sum);
        }
    "#;
    assert!(run(src).is_ok(), "{}", run(src).unwrap_err());
}

// `init` runs once, the condition gates each iteration, and `update` advances
// the counter: the accumulated sum must be exactly 0+1+...+9 = 45.
#[test]
fn test_for_accumulates_sum() {
    let env = run_main_and_inspect(
        "void main() { int sum = 0; for (int i = 0; i < 10; i = i + 1) { sum = sum + i; } }",
    );
    assert_eq!(env.get("sum"), Some(&Value::Int(45)));
}

// The variable declared in `init` is loop-local and must be gone afterwards,
// while assignments to outer variables persist.
#[test]
fn test_for_loop_var_removed_after_loop() {
    let env = run_main_and_inspect(
        "void main() { int sum = 0; for (int i = 0; i < 3; i = i + 1) { sum = sum + 1; } }",
    );
    assert_eq!(env.get("i"), None, "loop variable must not survive the loop");
    assert_eq!(env.get("sum"), Some(&Value::Int(3)));
}

// When the condition is false on entry the body never executes.
#[test]
fn test_for_zero_iterations() {
    let env = run_main_and_inspect(
        "void main() { int x = 7; for (int i = 0; i < 0; i = i + 1) { x = 0; } }",
    );
    assert_eq!(env.get("x"), Some(&Value::Int(7)));
}

// Nested loops: the inner body runs `outer * inner` times.
#[test]
fn test_for_nested() {
    let env = run_main_and_inspect(
        "void main() { int c = 0; \
         for (int i = 0; i < 3; i = i + 1) { \
             for (int j = 0; j < 4; j = j + 1) { c = c + 1; } } }",
    );
    assert_eq!(env.get("c"), Some(&Value::Int(12)));
}

// An assignment-form `init` reuses a variable from the enclosing scope, which
// therefore keeps the value that first failed the loop condition.
#[test]
fn test_for_assign_init_persists_outer_var() {
    let env = run_main_and_inspect(
        "void main() { int i = 0; int last = 0; \
         for (i = 1; i < 5; i = i + 1) { last = i; } }",
    );
    assert_eq!(env.get("i"), Some(&Value::Int(5)));
    assert_eq!(env.get("last"), Some(&Value::Int(4)));
}

// A `return` inside the body short-circuits the entire loop. Here the function
// returns the first `i` whose square is at least 10, i.e. 4.
#[test]
fn test_for_return_stops_iteration() {
    let env = run_main_and_inspect(
        r#"
        int first() {
            for (int i = 0; i < 100; i = i + 1) {
                if i * i >= 10 { return i; }
            }
            return -1;
        }
        void main() { int r = first(); }
        "#,
    );
    assert_eq!(env.get("r"), Some(&Value::Int(4)));
}
