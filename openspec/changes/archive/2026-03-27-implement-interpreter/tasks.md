## 1. Module Scaffold

- [x] 1.1 Create `src/interpreter/` directory with empty `mod.rs`
- [x] 1.2 Create `src/interpreter/value.rs` with the `Value` enum (`Int`, `Float`, `Bool`, `Str`, `Array`, `Void`) and a `Display` impl
- [x] 1.3 Create `src/interpreter/env.rs` with `RuntimeEnv` struct holding variable bindings (`HashMap<String, Value>`) and function registry (`HashMap<String, FunDecl<Type>>`), plus snapshot/restore methods
- [x] 1.4 Create `src/interpreter/eval_expr.rs` with a stub `eval_expr` function signature
- [x] 1.5 Create `src/interpreter/exec_stmt.rs` with a stub `exec_stmt` function signature
- [x] 1.6 Export the interpreter module from `src/lib.rs`

## 2. Value and Environment

- [x] 2.1 Implement `RuntimeEnv::new()` that initializes empty variable and function maps
- [x] 2.2 Implement `RuntimeEnv::get_var` and `RuntimeEnv::set_var` for variable lookup and update
- [x] 2.3 Implement `RuntimeEnv::declare_var` for introducing a new binding (used by `Decl`)
- [x] 2.4 Implement `RuntimeEnv::snapshot` and `RuntimeEnv::restore` for scoped execution
- [x] 2.5 Implement `RuntimeEnv::register_fn` and `RuntimeEnv::get_fn` for function lookup
- [x] 2.6 Define `RuntimeError` struct with a `message: String` field and implement `std::error::Error` for it

## 3. Expression Evaluation

- [x] 3.1 Implement literal evaluation: map each `Literal` variant to the corresponding `Value` variant
- [x] 3.2 Implement identifier evaluation: look up name in `RuntimeEnv`, return error if not found
- [x] 3.3 Implement unary negation (`Expr::Neg`) for `Int` and `Float` values
- [x] 3.4 Implement binary arithmetic (`Add`, `Sub`, `Mul`, `Div`) with int/float coercion rules
- [x] 3.5 Implement relational comparisons (`Lt`, `Le`, `Gt`, `Ge`) returning `Value::Bool`, with int/float coercion
- [x] 3.6 Implement equality comparisons (`Eq`, `Ne`) returning `Value::Bool`
- [x] 3.7 Implement logical `Not` on `Value::Bool`
- [x] 3.8 Implement short-circuit `And`: evaluate left; skip right if left is `false`
- [x] 3.9 Implement short-circuit `Or`: evaluate left; skip right if left is `true`
- [x] 3.10 Implement array literal evaluation: evaluate each element and collect into `Value::Array`
- [x] 3.11 Implement array index evaluation: bounds-check and return element, or `RuntimeError` if out of bounds

## 4. Statement Execution

- [x] 4.1 Define `ExecResult = Result<Option<Value>, RuntimeError>` type alias (where `Some(v)` is an early return)
- [x] 4.2 Implement `Statement::Decl`: evaluate init expression and bind name in env
- [x] 4.3 Implement `Statement::Assign` for simple identifier lvalues: evaluate value, update env binding
- [x] 4.4 Implement `Statement::Assign` for single-level array index lvalues: update element in-place
- [x] 4.5 Implement `Statement::Assign` for nested array index lvalues (e.g., `arr[i][j] = v`)
- [x] 4.6 Implement `Statement::Block`: snapshot env, execute each statement in order, propagate early return, restore env
- [x] 4.7 Implement `Statement::If`: evaluate condition, execute appropriate branch
- [x] 4.8 Implement `Statement::While`: loop while condition is `true`, propagate early return from body
- [x] 4.9 Implement `Statement::Return(Some(expr))`: evaluate expr, return early-return signal
- [x] 4.10 Implement `Statement::Return(None)`: return early-return signal with `Value::Void`
- [x] 4.11 Implement `Statement::Call` (statement form): dispatch to function, discard return value

## 5. Function Dispatch

- [x] 5.1 Implement `eval_call(name, args, env)` helper: evaluate args, snapshot env, bind params, exec body, restore env, unwrap return value
- [x] 5.2 Handle `print` as a built-in in `eval_call`: format `Value` with `Display`, write to stdout with newline, return `Value::Void`
- [x] 5.3 Return `RuntimeError` from `eval_call` when the function name is not found in the env and is not a built-in
- [x] 5.4 Return `RuntimeError` from `eval_call` when the argument count does not match the parameter count
- [x] 5.5 Wire `Expr::Call` in `eval_expr` to call `eval_call`
- [x] 5.6 Wire `Statement::Call` in `exec_stmt` to call `eval_call` and discard the result

## 6. Interpreter Entry Point

- [x] 6.1 Implement `interpret(program: &CheckedProgram) -> Result<(), RuntimeError>` in `mod.rs`
- [x] 6.2 Register all user-defined function declarations into `RuntimeEnv` at startup
- [x] 6.3 Locate the `main` function; return `RuntimeError` if it does not exist
- [x] 6.4 Call `eval_call("main", [], env)` to start execution
- [x] 6.5 Wire `main.rs` to run the full pipeline: parse source → type-check → interpret, printing errors to stderr

## 7. Tests

- [x] 7.1 Create `tests/interpreter.rs` with a helper that parses, type-checks, and interprets a source string
- [x] 7.2 Add test: empty `void main()` returns `Ok(())`
- [x] 7.3 Add test: arithmetic expressions produce correct `Value` (integer and float)
- [x] 7.4 Add test: `if/else` branches execute correctly based on condition
- [x] 7.5 Add test: `while` loop runs the expected number of iterations
- [x] 7.6 Add test: recursive function (e.g., factorial) returns the correct result
- [x] 7.7 Add test: array declaration, element assignment, and element read work correctly
- [x] 7.8 Add test: nested array element assignment (`arr[i][j] = v`) works correctly
- [x] 7.9 Add test: `print` produces output on stdout (use output capture or integration fixture)
- [x] 7.10 Add test: out-of-bounds array access returns `RuntimeError`
- [x] 7.11 Add test: calling undefined function returns `RuntimeError`
- [x] 7.12 Add fixture `.minic` programs in `tests/fixtures/` for end-to-end interpreter tests
