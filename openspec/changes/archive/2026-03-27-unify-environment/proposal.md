## Why

The codebase has two separate environment structs (`Environment<Type>` and `RuntimeEnv`) that duplicate scoping logic, and the type checker carries ad-hoc special-casing for polymorphic stdlib functions like `print`. Unifying them into a single `Environment<V>` and introducing `Type::Any` eliminates the duplication, removes the hacks, and simplifies the public API of both `type_check` and `interpret`.

## What Changes

- **BREAKING** `type_check(program, &registry)` → `type_check(program)` — registry no longer a parameter
- **BREAKING** `interpret(program, registry)` → `interpret(program)` — registry no longer a parameter
- Add `Type::Any` variant — matches any type in `types_compatible`; used for polymorphic stdlib params (e.g. `print`)
- Add `Type::Fn(Vec<Type>, Box<Type>)` variant — represents function bindings in the type environment
- Add `Value::Fn(FnValue)` variant (`FnValue = UserDefined(CheckedFunDecl) | Native(NativeFn)`) — represents function bindings in the runtime environment
- Unify `Environment<Type>` and `RuntimeEnv` into a single `Environment<V>` struct — variables and functions live in the same map; scoping operations (`snapshot`, `restore`, `names`, `remove_new`) are shared
- Remove `RuntimeEnv` entirely — interpreter uses `Environment<Value>` directly
- Remove `NativeEntry::skip_arg_type_check` — replaced by `Type::Any`
- Remove `NativeRegistry` parameter from public API — both `type_check` and `interpret` call `NativeRegistry::default()` internally and register natives into the environment at startup

## Capabilities

### New Capabilities

- `unified-environment`: Single parametric `Environment<V>` with unified variable and function storage, shared scoping primitives, and support for `Type::Any` and `Value::Fn`

### Modified Capabilities

- `type-checker`: `type_check` signature changes; native function lookup now goes through the unified environment; `Type::Any` is a valid parameter type
- `function-dispatch`: Functions (user-defined and native) are stored and looked up uniformly in `Environment<V>`; `Value::Fn` is the runtime representation
- `interpreter-core`: `interpret` signature changes; `RuntimeEnv` removed; interpreter uses `Environment<Value>` directly

## Impact

- `src/ir/ast.rs` — add `Type::Any`, `Type::Fn`
- `src/interpreter/value.rs` — add `Value::Fn(FnValue)`, `FnValue` enum
- `src/environment/mod.rs` — rewrite to unified `Environment<V>` (remove function-signature map)
- `src/interpreter/env.rs` — remove entirely
- `src/interpreter/mod.rs` — `interpret` takes no registry; builds `Environment<Value>` internally
- `src/interpreter/eval_expr.rs` — look up functions via `env.get()` returning `Value::Fn`
- `src/interpreter/exec_stmt.rs` — minor updates for new env API
- `src/semantic/type_checker.rs` — `type_check` takes no registry; registers natives as `Type::Fn` bindings; `Type::Any` replaces `skip_arg_type_check`
- `src/stdlib/mod.rs` — remove `skip_arg_type_check` from `NativeEntry`; use `Type::Any` for `print` params
- `src/main.rs`, `tests/interpreter.rs`, `tests/type_checker.rs` — drop registry construction and passing
