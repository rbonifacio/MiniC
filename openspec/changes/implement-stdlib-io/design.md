## Context

MiniC currently has one built-in function, `print`, wired via ad-hoc `if name == "print"` guards in two places: `type_checker.rs` (`check_call` and `Expr::Call` inference) and `eval_expr.rs` (`eval_call`). This pattern does not scale — each new built-in would require editing the core evaluation loop.

The goal is to replace this with a `NativeRegistry` that lives in a new `src/stdlib/` module. Both the type checker and the interpreter consult the registry by name; adding a new stdlib function requires only registering it in one place, with no changes to the evaluation logic.

## Goals / Non-Goals

**Goals:**
- Define a `NativeFn` type and `NativeRegistry` struct that maps names → (type signature, Rust implementation).
- Implement `readInt`, `readFloat`, `readString`, `print` in `src/stdlib/io.rs`.
- Implement `pow`, `sqrt` in `src/stdlib/math.rs`.
- Update the type checker to consult the registry for built-in signatures.
- Update the interpreter's `eval_call` to dispatch to the registry before user-defined functions.
- Remove all existing ad-hoc `print` special-cases.

**Non-Goals:**
- Variadic functions or functions with optional parameters — all built-ins have fixed arity.
- User-defined functions that call into native code (FFI) — only interpreter-internal dispatch.
- Stdin line editing or formatting options — basic `read_line` / `parse` is sufficient.
- Error recovery on bad input (e.g., non-numeric stdin for `readInt`) — a `RuntimeError` is acceptable.

## Decisions

### D1: `NativeFn` as a Rust function pointer

**Decision:** Define:
```rust
pub type NativeFn = fn(Vec<Value>) -> Result<Value, RuntimeError>;
```

**Rationale:** A plain function pointer is the simplest representation — no heap allocation, no closures needed for pure IO/math functions. All stdlib functions are stateless with respect to the MiniC runtime.

**Alternatives considered:**
- *`Box<dyn Fn(...)>`*: Supports closures and stateful built-ins, but adds unnecessary complexity and allocation for this use case.
- *Enum of built-in kinds*: Avoids function pointers but requires a match arm per built-in everywhere — worse than what we're replacing.

---

### D2: `NativeRegistry` owns both signature and implementation

**Decision:** A single `NativeRegistry` struct holds both the MiniC type signature (for the type checker) and the Rust `NativeFn` (for the interpreter):

```rust
pub struct NativeEntry {
    pub params: Vec<Type>,
    pub return_type: Type,
    pub func: NativeFn,
}

pub struct NativeRegistry {
    entries: HashMap<String, NativeEntry>,
}
```

**Rationale:** Keeping signature and implementation together guarantees they cannot drift. The type checker only reads `params`/`return_type`; the interpreter only calls `func`. A single `NativeRegistry::default()` call initializes both consumers.

**Alternatives considered:**
- *Two separate maps* (one for types, one for fns): Duplicates registration, risks type/impl mismatch.
- *Register in `Environment`*: Mixes semantic and runtime concerns; `Environment` is already parameterized over `T` for the type checker.

---

### D3: Registry passed to `eval_call` and `type_checker` as a shared reference

**Decision:** `interpret(program, registry)` takes a `&NativeRegistry`. The type checker `type_check(program, registry)` also takes a `&NativeRegistry`. A default registry (all stdlib functions) is constructed in `main.rs` and passed to both.

**Rationale:** Passing the registry explicitly (rather than using a global or thread-local) keeps the design testable — tests can construct a minimal registry or inject mock functions.

**Alternatives considered:**
- *`lazy_static` global registry*: Works, but makes unit testing harder and hides the dependency.
- *Registry stored on `RuntimeEnv`*: Conflates the runtime variable scope with the stdlib dispatch table.

---

### D4: Native functions checked before user-defined functions in dispatch

**Decision:** In `eval_call`, check the `NativeRegistry` first; if not found there, check user-defined functions in `RuntimeEnv`. The type checker mirrors this order.

**Rationale:** Native names (`print`, `sqrt`, etc.) are reserved — a user cannot shadow them. Checking native first enforces this reservation cheaply.

**Alternatives considered:**
- *User-defined functions first*: Would allow shadowing builtins, which is surprising behavior and complicates the type system.

---

### D5: `print` accepts `Value::Array` and formats it recursively

**Decision:** `print`'s `Display` impl (already on `Value`) covers all types including arrays. No special formatting logic is needed beyond `println!("{}", val)`.

**Rationale:** Consistent with the existing `Value::Display` impl introduced in the interpreter change. Users get predictable output for all types.

---

### D6: `readInt` / `readFloat` / `readString` use line-buffered stdin

**Decision:** Each read function reads one line from `stdin`, trims whitespace, and parses. On parse failure, returns `RuntimeError`.

**Rationale:** Simple and predictable. No prompt is printed (MiniC programs can call `print` first). Matches the behavior of equivalent functions in introductory languages (Pascal `readln`, Java `Scanner.nextInt`).

---

### D7: `pow` and `sqrt` operate on `Float`; `Int` arguments are coerced

**Decision:** `pow(base, exp)` and `sqrt(x)` accept `Value::Float`. If an `Int` is passed, it is coerced to `Float` before the Rust `f64` operation. Both return `Value::Float`.

**Rationale:** Math functions naturally produce floating-point results. Coercing `Int` inputs (consistent with how the interpreter handles mixed arithmetic) avoids requiring the caller to write `pow(2.0, 3.0)` for integer inputs. The type checker registers `pow` as `(Float, Float) → Float` and `sqrt` as `(Float) → Float`, relying on the existing `Int ↔ Float` type compatibility rule to accept integer arguments at call sites.

## Risks / Trade-offs

- **stdin blocking in tests**: `readInt` / `readFloat` / `readString` block on stdin, making automated tests that exercise them awkward. → *Mitigation*: Test IO functions with a dedicated helper that feeds a mock stdin, or test them only via manual integration fixtures. The registry design (D3) allows injecting alternative implementations in test builds.

- **`readInt` parse failure is a hard error**: Non-numeric input produces a `RuntimeError` with no recovery. → *Mitigation*: Document clearly; acceptable for a teaching language. Future work could add `tryReadInt` returning `bool`.

- **`pow` with negative exponents or `sqrt` of negative numbers**: Rust `f64::powf` and `f64::sqrt` return `f64::INFINITY` or `f64::NAN` rather than erroring. → *Mitigation*: Let the Rust semantics stand for now; document that MiniC math follows IEEE 754. A `NaN` check could be added later.

- **Signature change to `type_check` and `interpret`**: Adding a `&NativeRegistry` parameter is a breaking change to the public API of `type_check` and `interpret`. → *Mitigation*: These are library functions; callers (only `main.rs` and tests) are updated in the same change.

## Migration Plan

1. Add `src/stdlib/` module with registry, IO, and math.
2. Update `type_check` signature to accept `&NativeRegistry`; replace `print` special-cases with registry lookups.
3. Update `interpret` signature to accept `&NativeRegistry`; replace `eval_call` `print` guard with registry dispatch.
4. Update `main.rs` to construct `NativeRegistry::default()` and pass it to both.
5. Update all tests that call `type_check` or `interpret` directly to pass the registry.
6. No rollback concern — this is an additive change with a clean internal API break.

## Open Questions

- Should `readString` strip only the trailing newline, or also leading/trailing whitespace? (Current decision: trim both ends for consistency with `readInt`/`readFloat`.)
- Should `print` be renamed to `println` to clarify that it appends a newline? (Current decision: keep `print` for familiarity; behavior is already documented in specs.)
