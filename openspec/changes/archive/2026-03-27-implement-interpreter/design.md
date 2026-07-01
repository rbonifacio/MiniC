## Context

MiniC has a complete frontend: a `nom`-based parser produces an `UncheckedProgram`, and the type checker produces a `CheckedProgram` — an AST fully decorated with `Type` at every expression node. No execution stage exists today.

The AST is already parameterized (`Program<Ty>`, `ExprD<Ty>`, `StatementD<Ty>`), so the interpreter can operate directly on `CheckedProgram` without touching the parser or type checker. The existing `Environment` in `src/environment/` tracks type-level bindings; the interpreter needs a parallel runtime environment for values.

## Goals / Non-Goals

**Goals:**
- Implement a tree-walking interpreter over `CheckedProgram`.
- Represent all MiniC runtime values in a single `Value` enum.
- Execute all expression and statement forms defined in the AST.
- Support function calls including recursion, with a built-in `print` function.
- Wire the full pipeline (`parse → type-check → interpret`) in `main.rs`.
- Cover the interpreter with integration tests.

**Non-Goals:**
- Code generation (LLVM IR, bytecode, assembly) — out of scope.
- Error recovery or multiple error reporting — a single `RuntimeError` on first failure is sufficient.
- Closures, higher-order functions, or first-class functions — not in the language.
- Optimization (constant folding, inlining) — not needed for a teaching interpreter.
- I/O beyond `print` (file I/O, stdin) — out of scope.

## Decisions

### D1: Tree-walking interpreter over a separate IR

**Decision:** Walk the `CheckedProgram` AST directly, not a separate bytecode or IR.

**Rationale:** MiniC is a small language used for learning. A tree-walker is the simplest correct implementation — no extra data structures, no lowering pass, no compilation step. Performance is not a requirement.

**Alternatives considered:**
- *Bytecode VM*: Faster execution but significantly more complexity (compiler to bytecode + VM loop). Unjustified for this scope.
- *LLVM codegen*: Maximum performance but requires `inkwell`/`llvm-sys`, large dependency, steep learning curve.

---

### D2: `Value` enum as the runtime representation

**Decision:** Define a single `Value` enum in `src/interpreter/value.rs`:

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Void,
}
```

**Rationale:** Mirrors the MiniC type system 1-to-1. `Void` is needed as the return value of `void` functions and `print`. Arrays are heap-allocated `Vec<Value>` — simple and correct; no sharing semantics are needed since MiniC has no references or aliasing.

**Alternatives considered:**
- *`Box<dyn Any>`*: Dynamically typed, loses compile-time exhaustiveness checking.
- *Tagged union via `union`*: Unsafe, no benefit over an enum.

---

### D3: Return values via a `ControlFlow` / signal type

**Decision:** Statement execution returns `Result<Option<Value>, RuntimeError>` where `Some(v)` signals an early return and `None` means normal completion (fall-through).

```rust
type ExecResult = Result<Option<Value>, RuntimeError>;
```

**Rationale:** Return statements inside loops or nested blocks must unwind the call stack back to the function boundary. Propagating an `Option<Value>` through the `?` operator on `Result` is idiomatic Rust and avoids panics or non-local jumps.

**Alternatives considered:**
- *`ControlFlow<Value, ()>` from `std::ops`*: Slightly more expressive but harder to combine with `Result`. The `Option` approach is simpler.
- *Exception via `panic!`*: Non-idiomatic, untestable, and conflates runtime errors with returns.

---

### D4: Separate runtime environment (not reuse `src/environment/`)

**Decision:** Implement a new `RuntimeEnv` in `src/interpreter/env.rs` that maps `String → Value` for variables and `String → FunDecl<Type>` for functions.

**Rationale:** The existing `Environment` maps names to `Type`, which is what the type checker needs. At runtime we need values, not types. Reusing the same struct would require generalizing it in ways that complicate the type checker. A separate, purpose-built runtime environment is cleaner.

**Scope management:** Mirror the existing snapshot/restore pattern — take a snapshot before entering a block or function, restore it on exit. This correctly implements lexical scoping without a full linked-environment chain.

---

### D5: Built-in `print` handled by name in function dispatch

**Decision:** In the function dispatch logic, check `if name == "print"` before looking up the function in the environment, and handle it inline (format the argument with `Display` and write to stdout).

**Rationale:** `print` is not a user-defined function and has no `FunDecl` in the AST. Intercepting it by name in the dispatch path is the simplest approach with zero AST changes. The type checker already special-cases `print`; the interpreter follows the same pattern.

**Alternatives considered:**
- *Register `print` as a `FunDecl` in the environment at startup*: Would require either a fake AST node or a separate `Builtin` variant in `Value`. Unnecessary complexity.

---

### D6: Module layout under `src/interpreter/`

**Decision:**

```
src/interpreter/
├── mod.rs          — pub fn interpret(program: &CheckedProgram) entry point
├── value.rs        — Value enum + Display impl
├── env.rs          — RuntimeEnv (variables + functions, snapshot/restore)
├── eval_expr.rs    — fn eval_expr(expr: &CheckedExpr, env: &mut RuntimeEnv) -> Result<Value, RuntimeError>
└── exec_stmt.rs    — fn exec_stmt(stmt: &CheckedStatement, env: &mut RuntimeEnv) -> ExecResult
```

**Rationale:** Matches the existing module split (parser/, semantic/, environment/). Each file has a single, clear responsibility. `eval_expr` and `exec_stmt` are mutually dependent (call expressions invoke `exec_stmt` via function dispatch), so they live in the same crate and call each other directly.

## Risks / Trade-offs

- **Array mutation semantics**: MiniC array assignment (`arr[i] = v`) requires mutating a `Vec<Value>` in place. Since `Value::Array(Vec<Value>)` is owned, assigning into a nested array (e.g., `arr[i][j] = v`) requires careful re-assembly of the value chain. This is fiddly but correct as long as assignments traverse the lvalue path and rebuild from the inside out. → *Mitigation*: Implement a dedicated `assign_lvalue` helper that handles the multi-level indexing case explicitly, with a test covering `arr[i][j] = v`.

- **Recursion depth**: The interpreter is recursive in Rust (each function call is a Rust stack frame). Deep MiniC recursion will hit Rust's stack limit. → *Mitigation*: Acceptable for a teaching interpreter; document the limitation. If needed, increase stack size via `RUST_MIN_STACK`.

- **int/float coercion**: The type checker allows `int + float → float`. The interpreter must replicate this coercion at runtime in `eval_expr` to avoid a mismatch panic. → *Mitigation*: Handle all mixed-type arithmetic cases explicitly in the `Value` arithmetic helpers.

## Open Questions

- Should `print` accept any `Value` type (including arrays), or only scalars? The type checker currently does not validate `print` argument types — the interpreter should decide a reasonable behavior (e.g., print array elements comma-separated).
- Should `main` be allowed to return a non-zero exit code (via `return 0;` style)? Currently `main` is `void`; this is fine as-is but worth confirming.
