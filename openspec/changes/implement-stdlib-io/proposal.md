## Why

MiniC programs currently have no way to read input or perform math operations beyond basic arithmetic — and the one existing built-in (`print`) is wired in through ad-hoc name checks scattered across the type checker and interpreter. Adding `readInt`, `readFloat`, `readString`, `pow`, and `sqrt` requires a principled, extensible mechanism for native (Rust-implemented) functions so that the stdlib can grow without touching the core evaluation logic each time.

## What Changes

- Introduce a `NativeFn` type and a `NativeRegistry` that maps function names to Rust implementations and their MiniC type signatures.
- Register all native functions in one place (`src/stdlib/mod.rs`); both the type checker and interpreter consult this registry.
- Remove the existing ad-hoc `if name == "print"` checks from the type checker and interpreter, replacing them with registry lookups. **BREAKING** (internal API only — no MiniC source compatibility impact).
- Add `src/stdlib/io.rs`: native implementations of `print`, `readInt`, `readFloat`, `readString`.
- Add `src/stdlib/math.rs`: native implementations of `pow` and `sqrt`.
- Export `pub mod stdlib` from `src/lib.rs`.

## Capabilities

### New Capabilities

- `native-function-registry`: The `NativeRegistry` infrastructure — `NativeFn` type alias, the registry struct, registration API, and the lookup contract used by both the type checker and interpreter.
- `stdlib-io`: Built-in IO functions: `print(value)` → void (any type), `readInt()` → int, `readFloat()` → float, `readString()` → str. Implemented in Rust using `std::io`.
- `stdlib-math`: Built-in math functions: `pow(base, exp)` → float, `sqrt(x)` → float. Implemented in Rust using `f64` primitives.

### Modified Capabilities

- `function-dispatch`: Dispatch must consult the `NativeRegistry` before user-defined functions. The existing ad-hoc `print` special-case is removed and replaced by the registry lookup.
- `interpreter-core`: The interpreter's `RuntimeEnv` no longer owns built-in logic; it delegates to the shared `NativeRegistry`.

## Impact

- **New module**: `src/stdlib/` with `mod.rs`, `io.rs`, `math.rs`.
- **Modified**: `src/interpreter/eval_expr.rs` — `eval_call` delegates to `NativeRegistry` instead of `if name == "print"`.
- **Modified**: `src/semantic/type_checker.rs` — `check_call` and `Expr::Call` type inference delegate to `NativeRegistry` for signature lookup instead of special-casing `print`.
- **Modified**: `src/interpreter/mod.rs` — `interpret()` initializes and passes the `NativeRegistry`.
- **Modified**: `src/lib.rs` — export `stdlib` module.
- **No AST changes** — the native dispatch is entirely in the interpreter/type checker layer.
- **Dependencies**: No new crates; `std::io` and `f64` math are sufficient.
- **Tests**: New unit tests for each stdlib function; update existing interpreter tests that relied on `print`.
