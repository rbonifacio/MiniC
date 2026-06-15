## MODIFIED Requirements

### Requirement: User-defined function call
The interpreter SHALL dispatch calls for user-defined functions by:
1. Evaluating all argument expressions in order.
2. Looking up the callee name in `Environment<Value>` via `env.get(name)`, expecting `Value::Fn(FnValue::UserDefined(decl))`.
3. Taking a snapshot of the current environment.
4. Binding each parameter name to its evaluated argument value.
5. Executing the function body.
6. Restoring the environment snapshot.
7. Returning the value carried by an early-return signal, or `Value::Void` if the body completes without a return.

#### Scenario: Function returns computed value
- **WHEN** a function `int add(int a, int b) { return a + b; }` is called as `add(3, 4)`
- **THEN** the result SHALL be `Value::Int(7)`

#### Scenario: Function with no return produces Void
- **WHEN** a `void` function body completes without a `return` statement
- **THEN** the result SHALL be `Value::Void`

#### Scenario: Environment is restored after function call
- **WHEN** a function binds a parameter `x` and the call returns
- **THEN** any variable named `x` in the caller's environment SHALL retain its pre-call value

---

### Requirement: Recursive function call
The interpreter SHALL support recursive function calls. Each invocation SHALL have its own independent parameter bindings established via snapshot/restore.

#### Scenario: Factorial via recursion
- **WHEN** a recursive `int factorial(int n)` function is called with `n = 5`
- **THEN** the result SHALL be `Value::Int(120)`

---

### Requirement: Undefined function error
The interpreter SHALL return a `RuntimeError` when `env.get(name)` returns `None` or a non-`Fn` value for a call target.

#### Scenario: Call to undefined function
- **WHEN** `foo(1)` is called and no binding named `foo` exists in the environment
- **THEN** the interpreter SHALL return `Err(RuntimeError)` identifying `foo`

---

### Requirement: Argument count mismatch error
The interpreter SHALL return a `RuntimeError` when the number of evaluated arguments does not match the number of parameters in the `FnValue`.

#### Scenario: Too few arguments
- **WHEN** a function expecting two parameters is called with one argument
- **THEN** the interpreter SHALL return `Err(RuntimeError)`

---

### Requirement: Native stdlib dispatch
The interpreter SHALL dispatch calls to native stdlib functions by looking up `Value::Fn(FnValue::Native(f))` in the environment and calling `f(args)`. Native function calls use the same `env.get(name)` path as user-defined functions — there is no separate registry lookup at call time.

#### Scenario: print via native dispatch
- **WHEN** `print(42)` is called
- **THEN** `env.get("print")` SHALL return `Value::Fn(FnValue::Native(...))`, `"42\n"` SHALL be written to stdout, and `Value::Void` SHALL be returned

#### Scenario: sqrt via native dispatch
- **WHEN** `sqrt(4)` is called
- **THEN** `env.get("sqrt")` SHALL return the native sqrt binding and the result SHALL be `Value::Float(2.0)`

#### Scenario: pow via native dispatch
- **WHEN** `pow(2, 10)` is called
- **THEN** the result SHALL be `Value::Float(1024.0)`
