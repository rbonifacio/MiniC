## 1. AST and Value Extensions

- [x] 1.1 Add `Type::Any` variant to `Type` enum in `src/ir/ast.rs`
- [x] 1.2 Add `FnValue` enum to `src/interpreter/value.rs`: `UserDefined(CheckedFunDecl)` and `Native(NativeFn)` variants
- [x] 1.3 Add `Value::Fn(FnValue)` variant to `Value` enum in `src/interpreter/value.rs`
- [x] 1.4 Update `Display` for `Value` to handle `Value::Fn` (render as `"<function>"` or similar)
- [x] 1.5 Update any exhaustive `Value` match arms in `eval_expr.rs`, `exec_stmt.rs`, or helpers that do not yet cover `Value::Fn`

## 2. Unified Environment

- [x] 2.1 Rewrite `src/environment/env.rs`: remove `function_signatures`, `function_declarations` fields; keep only `bindings: HashMap<String, V>`
- [x] 2.2 Replace `add_binding` / `lookup` with `declare(name, value)` and `get(name)` in `Environment<V>`
- [x] 2.3 Add `set(name, value) -> bool` (returns false if name not bound) to `Environment<V>`
- [x] 2.4 Rename `snapshot_bindings` / `restore_bindings` to `snapshot` / `restore` in `Environment<V>`
- [x] 2.5 Rename `var_names` / `remove_new_vars` to `names` / `remove_new` in `Environment<V>` (add if not present)
- [x] 2.6 Remove `add_function_signature`, `lookup_function_signature`, `add_function_declaration`, `lookup_function`, `clear_bindings` from `Environment<V>`
- [x] 2.7 Update `src/environment/mod.rs` re-exports to match the new API (remove `FuncSig` export)

## 3. Update NativeRegistry

- [x] 3.1 Remove `skip_arg_type_check` field from `NativeEntry` in `src/stdlib/mod.rs`
- [x] 3.2 Update `NativeRegistry::default()`: replace the `print` entry's placeholder param type with `Type::Any`
- [x] 3.3 Remove all `skip_arg_type_check` references from `eval_expr.rs` and `type_checker.rs`

## 4. Update Type Checker

- [x] 4.1 Remove `registry: &NativeRegistry` parameter from `type_check` and all internal helpers in `src/semantic/type_checker.rs`
- [x] 4.2 At the start of `type_check`: construct `NativeRegistry::default()`, register each native entry as `Type::Fun(params, return_type)` binding in the environment
- [x] 4.3 Register user-defined function signatures as `Type::Fun(params, return_type)` bindings (replacing `add_function_signature` calls)
- [x] 4.4 Replace `env.lookup_function_signature(name)` calls in `check_call` and `Expr::Call` type inference with `env.get(name)` matching on `Type::Fun`
- [x] 4.5 Update `types_compatible` to return `true` when the right-hand side is `Type::Any`
- [x] 4.6 Replace `env.clear_bindings()` with the clean-snapshot pattern: take a snapshot after registering all function bindings; restore to it before checking each function body
- [x] 4.7 Replace `env.add_binding` / `env.lookup` calls with `env.declare` / `env.get` throughout the type checker
- [x] 4.8 Replace `env.snapshot_bindings` / `env.restore_bindings` with `env.snapshot` / `env.restore`

## 5. Update Interpreter

- [x] 5.1 Delete `src/interpreter/env.rs` (`RuntimeEnv` struct)
- [x] 5.2 Update `src/interpreter/mod.rs`: change `interpret` signature to `interpret(program: &CheckedProgram) -> Result<(), RuntimeError>`; construct `NativeRegistry::default()` and `Environment<Value>` internally; register all functions (user-defined and native) as `Value::Fn` bindings
- [x] 5.3 Update `src/interpreter/eval_expr.rs`: replace `env: &mut RuntimeEnv` with `env: &mut Environment<Value>` in all function signatures
- [x] 5.4 Rewrite `eval_call` in `eval_expr.rs`: use `env.get(name)` and match on `Value::Fn(FnValue::UserDefined(...))` / `Value::Fn(FnValue::Native(...))` for dispatch; remove registry field access
- [x] 5.5 Update `src/interpreter/exec_stmt.rs`: replace `RuntimeEnv` with `Environment<Value>`; update `env.declare_var` / `env.get_var` / `env.set_var` calls to `env.declare` / `env.get` / `env.set`; update `env.var_names` / `env.remove_new_vars` to `env.names` / `env.remove_new`
- [x] 5.6 Update snapshot/restore calls in `eval_expr.rs` (function call scoping) to use `env.snapshot` / `env.restore`
- [x] 5.7 Remove `use crate::stdlib::NativeRegistry` and `RuntimeEnv` imports from all interpreter files

## 6. Update Call Sites

- [x] 6.1 Update `src/main.rs`: remove registry construction and parameter passing from both `type_check` and `interpret` calls
- [x] 6.2 Update `tests/interpreter.rs` `run()` helper: remove registry construction and parameters
- [x] 6.3 Update `tests/type_checker.rs` `parse_and_type_check()` helper: remove registry construction and parameter

## 7. Tests

- [x] 7.1 Verify all existing 119 tests pass after the refactor (`cargo test`)
- [x] 7.2 Add a type-checker test: `print` accepts `int`, `bool`, `str`, `float`, and `int[]` arguments without error
- [x] 7.3 Add a type-checker test: calling a stdlib function with wrong arity reports a type error (e.g. `sqrt()` with no args)
- [x] 7.4 Add an interpreter integration test: `readInt`, `readFloat`, `readString` are registered and reachable (type-check succeeds; runtime behavior tested by existing unit tests)
- [x] 7.5 Add an interpreter integration test: `pow(2.0, 3.0)` returns `8.0` via the unified dispatch path
