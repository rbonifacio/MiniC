# 7 — The Standard Library

MiniC comes with a small set of built-in functions available to every
program. This document describes them from a user perspective and then
explains how they are implemented and how to add new ones.

Note: `len(...)` and `contains(...)` are core language expressions in the
parser/type-checker/interpreter pipeline. They are not registered as native
functions in `NativeRegistry`.

---

## Built-in Functions

### I/O

#### `print(x)`

Signature: `(any) → void`

Prints its argument to standard output followed by a newline. Accepts any
MiniC value type.

```c
print(42)           -- prints: 42
print(3.14)         -- prints: 3.14
print(true)         -- prints: true
print("hello")      -- prints: hello
print([1, 2, 3])    -- prints: [1, 2, 3]
```

#### `readInt()`

Signature: `() → int`

Reads one line from standard input and parses it as an integer. Returns a
runtime error if the input is not a valid integer.

```c
int n = readInt()
```

#### `readFloat()`

Signature: `() → float`

Reads one line from standard input and parses it as a floating-point number.

```c
float x = readFloat()
```

#### `readString()`

Signature: `() → str`

Reads one line from standard input and returns it as a string (leading and
trailing whitespace trimmed).

```c
str name = readString()
```

---

### Math

#### `sqrt(x)`

Signature: `(float) → float`

Returns the square root of `x`. Accepts `int` or `float` arguments; always
returns `float`.

```c
float s = sqrt(16)      -- s = 4.0
float s = sqrt(2.0)     -- s ≈ 1.4142
```

#### `pow(base, exp)`

Signature: `(float, float) → float`

Returns `base` raised to the power `exp`. Accepts `int` or `float` arguments;
always returns `float`.

```c
float p = pow(2, 10)    -- p = 1024.0
float p = pow(2.0, 0.5) -- p ≈ 1.4142 (square root via pow)
```

---

## How the Registry Works

All built-in functions are registered in a `NativeRegistry` — a map from
function name to a `NativeEntry`. Each entry bundles two things:

- **The type signature** — parameter types and return type — used by the
  type checker to validate calls.
- **The Rust function** — the actual implementation — called by the
  interpreter at runtime.

```rust
struct NativeEntry {
    params:      Vec<Type>,  // MiniC parameter types
    return_type: Type,       // MiniC return type
    func:        NativeFn,   // fn(Vec<Value>) -> Result<Value, RuntimeError>
}
```

`NativeFn` is a *function pointer type* — a value that holds the address of a
Rust function with a specific signature. It is lightweight (just one pointer)
and requires no heap allocation.

Both the type checker and the interpreter call `NativeRegistry::default()` at
startup to get the same registry, so the type signature and implementation are
always in sync.

---

## Why `print` Accepts Any Type

`print` must work with integers, floats, booleans, strings, and arrays — any
type. Rather than adding special-case logic to the type checker, `print` is
registered with a parameter type of `Type::Any`. The type compatibility check
treats `Any` as matching everything, so `print(42)` and `print(true)` and
`print([1,2])` all pass type checking without any extra rules.

`Type::Any` only exists as a parameter type in native function registrations.
It is never inferred as the type of a variable or expression.

---

## How to Add a New Native Function

Adding a new built-in function to MiniC takes three steps.

### Step 1 — Implement the function in Rust

Create the Rust implementation in `src/stdlib/math.rs` or `src/stdlib/io.rs`
(or a new file). The signature must match `NativeFn`:
`fn(Vec<Value>) -> Result<Value, RuntimeError>`.

```rust
// Example: absolute value
pub fn abs_fn(args: Vec<Value>) -> Result<Value, RuntimeError> {
    match args.as_slice() {
        [Value::Int(n)]   => Ok(Value::Int(n.abs())),
        [Value::Float(f)] => Ok(Value::Float(f.abs())),
        _ => Err(RuntimeError::new(
            "abs: expected exactly one numeric argument"
        )),
    }
}
```

The function receives its arguments as a `Vec<Value>`. It must validate them
manually (the type checker ensures types match, but not arity at the Rust
level), then return a `Value` or a `RuntimeError`.

### Step 2 — Register it in the registry

Add a registration call inside `NativeRegistry::default()` in
`src/stdlib/mod.rs`:

```rust
r.register("abs", NativeEntry {
    params:      vec![Type::Float],  // MiniC type signature for type checking
    return_type: Type::Float,
    func:        math::abs_fn,       // the Rust function from Step 1
});
```

Use `Type::Any` in `params` if the function should accept multiple value
types (as `print` does).

### Step 3 — Done

The function is now available in every MiniC program. Both the type checker
and the interpreter pick it up automatically from the registry. No other files
need to be changed.

To test it, add a test in `tests/stdlib.rs` or `tests/interpreter.rs` — see
[08-testing.md](08-testing.md) for guidance.

---

**What to read next →** [08-testing.md](08-testing.md)
