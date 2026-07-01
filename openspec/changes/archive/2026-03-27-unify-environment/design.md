## Context

MiniC currently has two environment structs with overlapping responsibilities:

- `Environment<T>` (`src/environment/env.rs`): used by the type checker; holds variable bindings, function signatures (forward declarations), and function declarations in three separate maps
- `RuntimeEnv` (`src/interpreter/env.rs`): used by the interpreter; holds variable bindings, user-defined function bodies, and the `NativeRegistry` in three separate fields

Both implement scoping — but independently, duplicating the logic. The type checker also carries ad-hoc `if name == "print"` guards (now replaced by `skip_arg_type_check`) to handle polymorphic stdlib params. The `NativeRegistry` is passed as an explicit parameter to both `type_check` and `interpret`, leaking an internal concern into the public API.

Notably, `Type::Fun(Vec<Type>, Box<Type>)` already exists in the AST but is unused — native support for function types is already in place.

## Goals / Non-Goals

**Goals:**
- Single `Environment<V>` struct — one `HashMap<String, V>`, no separate function maps
- Variables and functions share the same bindings map; scoping operations are uniform
- `Type::Any` added to the AST — replaces `skip_arg_type_check` for polymorphic stdlib functions
- `Value::Fn(FnValue)` added — uniform runtime representation for user-defined and native functions
- `type_check(program)` and `interpret(program)` become parameter-free; registry is internal
- `RuntimeEnv` removed; `environment/env.rs` `function_signatures` and `function_declarations` maps removed

**Non-Goals:**
- First-class function values in the MiniC language (functions remain declaration-only)
- Scoped function declarations (all functions remain global)
- Changes to the parser or any AST node other than adding `Type::Any`

## Decisions

### 1. Functions in the unified bindings map

Functions are stored in the same `HashMap<String, V>` as variables. For the type checker, a function named `foo` is bound to `Type::Fun(param_types, return_type)`. For the interpreter, it is bound to `Value::Fn(FnValue::UserDefined(decl))` or `Value::Fn(FnValue::Native(f))`.

**Why:** Eliminates the separate `function_signatures` / `fns` / `registry` maps. Scoping operations (`snapshot`, `restore`, `remove_new`) apply uniformly to everything in the map.

**Concern — snapshot overhead:** On function call (snapshot/restore) and block exit (`remove_new`), function bindings are included in the snapshot. This is harmless: functions are registered once at startup and never reassigned. For a teaching language, the overhead is acceptable.

**Concern — `clear_bindings` in the type checker:** The type checker currently calls `clear_bindings()` before checking each function body to reset variable scope. With a unified map, "clear only variable bindings" is no longer straightforward. The replacement: after registering all function signatures, take a **clean snapshot** (contains only function bindings). Before checking each function body, restore to this clean snapshot, then add parameter bindings. This is semantically equivalent.

### 2. `Type::Any` for polymorphic stdlib params

`print` accepts one argument of any type. This is represented as `params: vec![Type::Any]` in the native registry entry. `types_compatible(_, Type::Any)` returns `true`. The `skip_arg_type_check` field is removed from `NativeEntry`.

**Why:** Encodes the polymorphism in the type system rather than in procedural guards. Any future polymorphic stdlib function follows the same pattern with no code changes needed in the type checker.

**Alternative considered:** Keep `skip_arg_type_check` as a flag on `NativeEntry`. Rejected: it is a workaround that doesn't generalize and leaks dispatch logic into the checker.

### 3. `Value::Fn(FnValue)` for runtime function representation

```rust
pub enum FnValue {
    UserDefined(CheckedFunDecl),
    Native(NativeFn),
}
```

`Value::Fn(FnValue)` is added to the `Value` enum. Both user-defined and native functions are looked up via `env.get(name)` and dispatched by matching on `Value::Fn`.

**Why:** Uniform dispatch — `eval_call` does a single `env.get(name)` and pattern-matches, with no separate registry lookup. `NativeRegistry` is still used as the source of stdlib definitions, but it is consumed during environment setup, not at call time.

### 4. `NativeRegistry` consumed at startup, not passed as parameter

Both `type_check` and `interpret` call `NativeRegistry::default()` internally at the start, register all entries into the environment, and proceed. The registry is not part of the public API.

**Why:** The stdlib is fixed for all current uses. Hiding the registry simplifies callers (`main.rs`, test helpers) and makes the API match the type checker's existing parameter-free interface for user-defined functions.

**Trade-off:** Cannot inject a custom registry from outside (e.g. for testing with a restricted stdlib). Mitigated by: native functions are individually unit-tested; end-to-end tests can test the full stdlib directly.

### 5. `Environment<V>` API

The unified struct exposes:
```rust
pub fn new() -> Self
pub fn get(&self, name: &str) -> Option<&V>
pub fn set(&mut self, name: &str, value: V) -> bool   // false if not found
pub fn declare(&mut self, name: &str, value: V)
pub fn snapshot(&self) -> HashMap<String, V>
pub fn restore(&mut self, snapshot: HashMap<String, V>)
pub fn names(&self) -> HashSet<String>
pub fn remove_new(&mut self, outer: &HashSet<String>)
```

`clear_bindings`, `add_function_signature`, `lookup_function_signature`, `add_function_declaration`, `lookup_function` are all removed.

## Risks / Trade-offs

**Snapshot includes function bindings** → Snapshot/restore on every function call copies function entries unnecessarily. Acceptable for a teaching language; could be optimised later by splitting into global/local layers.

**`Value::Fn` is not a MiniC language value** → Users cannot assign functions to variables in MiniC. `Value::Fn` existing in the enum means pattern-match exhaustiveness in `Display`, arithmetic, etc. must explicitly handle or reject it. All existing `Value` match arms that don't cover `Fn` will need a catch-all or explicit error arm.

**`Type::Any` must not leak into user programs** → The parser will not produce `Type::Any`; it is only created by the native registry. The type checker should propagate it only as a parameter type, never as an inferred expression type. This invariant is maintained by the registry-only construction path.

## Migration Plan

1. Add `Type::Any` to `ir/ast.rs`
2. Add `Value::Fn(FnValue)` and `FnValue` enum to `interpreter/value.rs`; update `Display` and any exhaustive matches
3. Rewrite `environment/env.rs` to the simplified `Environment<V>` API
4. Update type checker: remove `add_function_signature` / `lookup_function_signature` calls; register user functions as `Type::Fun` bindings; register natives from `NativeRegistry::default()` as `Type::Fun` bindings with `Type::Any` where applicable; use clean-snapshot pattern instead of `clear_bindings`
5. Update interpreter: remove `RuntimeEnv`; use `Environment<Value>` directly; register all functions as `Value::Fn` bindings at startup; update `eval_call` to use `env.get(name)` and match on `Value::Fn`
6. Remove `skip_arg_type_check` from `NativeEntry`; update `NativeRegistry::default()` to use `Type::Any`
7. Update `main.rs` and test helpers to drop registry parameter
8. Run full test suite; fix any exhaustiveness or type errors

## Open Questions

- Should `environment/env.rs` become the sole file in `src/environment/`, eliminating the `mod.rs` indirection, or keep the current module structure?
- Should `FnValue` live in `interpreter/value.rs` alongside `Value`, or in a separate `interpreter/fn_value.rs`?
