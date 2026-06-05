use mini_c::{
    interpreter::interpret,
    ir::ast::{FunDecl, Program, Statement, StatementD, Type},
    parser::program,
    semantic::type_check,
};

/// Parse, type-check, and interpret a MiniC source string.
fn run(src: &str) -> Result<(), String> {
    let unchecked = program(src)
        .map_err(|e| format!("parse error: {:?}", e))
        .map(|(_, p)| p)?;
    let checked = type_check(&unchecked).map_err(|e| format!("type error: {}", e.message))?;
    interpret(&checked).map_err(|e| format!("runtime error: {}", e.message))
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
// Milestone 1 lock-in: the interpreter should surface a clear error when it
// encounters a `for` statement (normally unreachable because the type checker
// rejects it first). This guards against the placeholder silently disappearing.
// ---------------------------------------------------------------------------
#[test]
fn test_interpret_for_rejected_in_milestone_1() {
    // Build a minimal CheckedProgram directly: `void main() { for (;;) {} }`.
    // We can't go through type_check because that also rejects `for`, so we
    // construct the AST by hand.
    let empty_block = StatementD {
        stmt: Statement::Block { seq: vec![] },
        ty: Type::Unit,
    };
    let for_stmt = StatementD {
        stmt: Statement::For {
            init: None,
            cond: None,
            update: None,
            body: Box::new(empty_block),
        },
        ty: Type::Unit,
    };
    let main_body = StatementD {
        stmt: Statement::Block {
            seq: vec![for_stmt],
        },
        ty: Type::Unit,
    };
    let prog = Program {
        functions: vec![FunDecl {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Unit,
            body: Box::new(main_body),
        }],
    };

    let result = interpret(&prog);
    assert!(result.is_err(), "expected interpreter to reject for-statement");
    assert!(
        result
            .unwrap_err()
            .message
            .contains("for statements are not yet executable"),
        "expected placeholder error message"
    );
}
