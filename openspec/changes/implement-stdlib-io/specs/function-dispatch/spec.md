## MODIFIED Requirements

### Requirement: Undefined function error
The interpreter SHALL return a `RuntimeError` when a call targets a function name that is not found in the `NativeRegistry` AND is not registered as a user-defined function in `RuntimeEnv`.

#### Scenario: Call to undefined function
- **WHEN** `foo(1)` is called and no function named `foo` exists in either the `NativeRegistry` or `RuntimeEnv`
- **THEN** the interpreter SHALL return `Err(RuntimeError)` identifying `foo`

---

### Requirement: Built-in print function
The interpreter SHALL dispatch `print` via the `NativeRegistry`. `print` SHALL accept a single argument of any `Value` type, format it using `Value`'s `Display` implementation, write it to standard output followed by a newline, and return `Value::Void`. The implementation SHALL reside in `src/stdlib/io.rs` and SHALL NOT be special-cased in `eval_call` by name.

#### Scenario: Print integer
- **WHEN** `print(42)` is called
- **THEN** `"42\n"` SHALL be written to stdout and `Value::Void` SHALL be returned

#### Scenario: Print boolean
- **WHEN** `print(true)` is called
- **THEN** `"true\n"` SHALL be written to stdout

#### Scenario: Print array
- **WHEN** `print([1, 2, 3])` is called
- **THEN** a human-readable representation of the array (e.g., `"[1, 2, 3]\n"`) SHALL be written to stdout

#### Scenario: Print string
- **WHEN** `print("hello")` is called
- **THEN** `"hello\n"` SHALL be written to stdout

## REMOVED Requirements

### Requirement: Built-in print function
**Reason**: Replaced by `NativeRegistry`-based dispatch. `print` is now registered as a `NativeFn` in `src/stdlib/io.rs` and dispatched through the registry like all other built-ins. The ad-hoc `if name == "print"` guard in `eval_call` is removed.
**Migration**: No MiniC source changes needed. Callers of `eval_call` in Rust no longer need to handle `print` specially; the registry handles it.
