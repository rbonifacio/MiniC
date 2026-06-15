## ADDED Requirements

### Requirement: User-defined function call
The interpreter SHALL dispatch `Expr::Call { name, args }` (and `Statement::Call`) for user-defined functions by:
1. Evaluating all argument expressions in order.
2. Taking a snapshot of the current environment.
3. Binding each parameter name to its evaluated argument value.
4. Executing the function body.
5. Restoring the environment snapshot.
6. Returning the value carried by an early-return signal, or `Value::Void` if the body completes without a return.

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
The interpreter SHALL support recursive function calls. Each invocation SHALL have its own independent parameter bindings.

#### Scenario: Factorial via recursion
- **WHEN** a recursive `int factorial(int n)` function is called with `n = 5`
- **THEN** the result SHALL be `Value::Int(120)`

---

### Requirement: Undefined function error
The interpreter SHALL return a `RuntimeError` when a call targets a function name that is not registered in the `RuntimeEnv` and is not a recognized built-in.

#### Scenario: Call to undefined function
- **WHEN** `foo(1)` is called and no function named `foo` exists
- **THEN** the interpreter SHALL return `Err(RuntimeError)` identifying `foo`

---

### Requirement: Argument count mismatch error
The interpreter SHALL return a `RuntimeError` when the number of evaluated arguments does not match the number of parameters declared for the function.

#### Scenario: Too few arguments
- **WHEN** a function expecting two parameters is called with one argument
- **THEN** the interpreter SHALL return `Err(RuntimeError)`

---

### Requirement: Built-in print function
The interpreter SHALL recognize `print` as a built-in function that accepts a single argument of any `Value` type, formats it as a human-readable string, and writes it to standard output followed by a newline. `print` SHALL return `Value::Void`.

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
