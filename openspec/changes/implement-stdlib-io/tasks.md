## 1. NativeRegistry Infrastructure

- [x] 1.1 Create `src/stdlib/` directory with empty `mod.rs`
- [x] 1.2 Define `NativeFn` type alias: `pub type NativeFn = fn(Vec<Value>) -> Result<Value, RuntimeError>` in `mod.rs`
- [x] 1.3 Define `NativeEntry` struct with `params: Vec<Type>`, `return_type: Type`, and `func: NativeFn` in `mod.rs`
- [x] 1.4 Define `NativeRegistry` struct wrapping `HashMap<String, NativeEntry>` in `mod.rs`
- [x] 1.5 Implement `NativeRegistry::new()` returning an empty registry
- [x] 1.6 Implement `NativeRegistry::register(name: &str, entry: NativeEntry)` to add a built-in
- [x] 1.7 Implement `NativeRegistry::lookup(name: &str) -> Option<&NativeEntry>` for dispatch
- [x] 1.8 Export `pub mod stdlib` from `src/lib.rs`

## 2. Stdlib IO Functions

- [x] 2.1 Create `src/stdlib/io.rs`
- [x] 2.2 Implement `print_fn(args: Vec<Value>) -> Result<Value, RuntimeError>`: format first arg with `Display`, write to stdout with newline, return `Value::Void`
- [x] 2.3 Implement `read_int_fn(args: Vec<Value>) -> Result<Value, RuntimeError>`: read one line from stdin, trim, parse as `i64`, return `Value::Int` or `RuntimeError`
- [x] 2.4 Implement `read_float_fn(args: Vec<Value>) -> Result<Value, RuntimeError>`: read one line from stdin, trim, parse as `f64`, return `Value::Float` or `RuntimeError`
- [x] 2.5 Implement `read_string_fn(args: Vec<Value>) -> Result<Value, RuntimeError>`: read one line from stdin, trim, return `Value::Str` or `RuntimeError` on EOF

## 3. Stdlib Math Functions

- [x] 3.1 Create `src/stdlib/math.rs`
- [x] 3.2 Implement `pow_fn(args: Vec<Value>) -> Result<Value, RuntimeError>`: coerce both args to `f64`, compute `f64::powf`, return `Value::Float`; error on wrong arity or non-numeric type
- [x] 3.3 Implement `sqrt_fn(args: Vec<Value>) -> Result<Value, RuntimeError>`: coerce arg to `f64`, compute `f64::sqrt`, return `Value::Float`; error on wrong arity or non-numeric type

## 4. Default Registry

- [x] 4.1 Implement `NativeRegistry::default()` in `mod.rs` that registers all stdlib functions with their MiniC type signatures:
  - `print`: params = `[Type::Unit]` (any), return = `Type::Unit`
  - `readInt`: params = `[]`, return = `Type::Int`
  - `readFloat`: params = `[]`, return = `Type::Float`
  - `readString`: params = `[]`, return = `Type::Str`
  - `pow`: params = `[Type::Float, Type::Float]`, return = `Type::Float`
  - `sqrt`: params = `[Type::Float]`, return = `Type::Float`

## 5. Update Type Checker

- [x] 5.1 Add `registry: &NativeRegistry` parameter to `type_check(program, registry)` signature
- [x] 5.2 In `check_call`: replace `if name == "print"` guard with `registry.lookup(name)` — return early with `Ok(())` after validating arg count against `entry.params.len()` (skip type check for `print` since it accepts any type)
- [x] 5.3 In `Expr::Call` type inference: replace `if name == "print"` guard with `registry.lookup(name)` — return `entry.return_type.clone()` on hit
- [x] 5.4 Update `main.rs` to construct `NativeRegistry::default()` and pass it to `type_check`
- [x] 5.5 Update all test helpers that call `type_check` to pass `&NativeRegistry::default()`

## 6. Update Interpreter

- [x] 6.1 Add `registry: &NativeRegistry` parameter to `interpret(program, registry)` signature
- [x] 6.2 Thread `registry` through to `eval_call` — update `eval_call` signature to `eval_call(name, args, env, registry)`
- [x] 6.3 In `eval_call`: replace `if name == "print"` guard with `registry.lookup(name)` — if found, validate arg count, call `entry.func(args)`, return result
- [x] 6.4 Remove the old `if name == "print"` block entirely from `eval_call`
- [x] 6.5 Update all internal call sites of `eval_call` inside `eval_expr.rs` and `exec_stmt.rs` to pass `registry`
- [x] 6.6 Update `main.rs` to pass `&NativeRegistry::default()` to `interpret`
- [x] 6.7 Update all test helpers that call `interpret` to pass `&NativeRegistry::default()`

## 7. Tests

- [x] 7.1 Add unit tests in `src/stdlib/io.rs` for `print_fn`, `read_int_fn`, `read_float_fn`, `read_string_fn` (use in-memory input where possible)
- [x] 7.2 Add unit tests in `src/stdlib/math.rs` for `pow_fn` and `sqrt_fn` covering int coercion, float inputs, edge cases (zero, negative exponents)
- [x] 7.3 Add unit test: `NativeRegistry::default()` contains entries for all 6 stdlib functions
- [x] 7.4 Add interpreter integration test: `sqrt(4)` returns `2.0` (int coercion)
- [x] 7.5 Add interpreter integration test: `pow(2, 10)` returns `1024.0`
- [x] 7.6 Add interpreter integration test: `print` still works end-to-end via registry (regression)
- [x] 7.7 Verify all existing 102 tests still pass after the signature changes
