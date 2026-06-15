## ADDED Requirements

### Requirement: Value representation
The interpreter SHALL represent all MiniC runtime values with a `Value` enum that covers every type in the MiniC type system: `Int(i64)`, `Float(f64)`, `Bool(bool)`, `Str(String)`, `Array(Vec<Value>)`, and `Void`. Every operation that produces a value SHALL return exactly one of these variants.

#### Scenario: Integer value round-trip
- **WHEN** a MiniC integer literal `42` is evaluated
- **THEN** the result SHALL be `Value::Int(42)`

#### Scenario: Array value contains homogeneous elements
- **WHEN** a MiniC array literal `[1, 2, 3]` is evaluated
- **THEN** the result SHALL be `Value::Array([Value::Int(1), Value::Int(2), Value::Int(3)])`

#### Scenario: Void is returned by void functions
- **WHEN** a `void` function returns normally (no explicit return value)
- **THEN** the result SHALL be `Value::Void`

---

### Requirement: Interpret entry point
The interpreter SHALL expose a public `interpret(program: &CheckedProgram) -> Result<(), RuntimeError>` function that locates the `main` function in the program, executes it, and returns `Ok(())` on success or a `RuntimeError` on failure.

#### Scenario: Successful execution of a minimal program
- **WHEN** `interpret` is called with a `CheckedProgram` containing a `void main()` function with an empty body
- **THEN** the function SHALL return `Ok(())`

#### Scenario: Missing main function
- **WHEN** `interpret` is called with a `CheckedProgram` that contains no function named `main`
- **THEN** the function SHALL return `Err(RuntimeError)` describing the missing entry point

---

### Requirement: Runtime error type
The interpreter SHALL define a `RuntimeError` type that carries a human-readable message describing the failure. Runtime errors SHALL be returned as `Err(RuntimeError)` and SHALL NOT cause a `panic!`.

#### Scenario: Runtime error on undefined variable
- **WHEN** the interpreter attempts to read a variable that is not in the current environment
- **THEN** it SHALL return `Err(RuntimeError)` with a message identifying the variable name

#### Scenario: Runtime error on array index out of bounds
- **WHEN** the interpreter evaluates an index expression where the index is outside the array bounds
- **THEN** it SHALL return `Err(RuntimeError)` with a message identifying the invalid index

---

### Requirement: Runtime environment
The interpreter SHALL maintain a `RuntimeEnv` that maps variable names to `Value` and function names to their `FunDecl<Type>`. The environment SHALL support snapshot and restore operations to implement lexical scoping for blocks and function calls.

#### Scenario: Variable binding is visible within its scope
- **WHEN** a variable `x` is declared inside a block `{ int x = 1; }`
- **THEN** `x` SHALL be accessible within that block and SHALL NOT be accessible after the block exits

#### Scenario: Snapshot restore removes inner bindings
- **WHEN** a snapshot is taken before entering a block, new variables are declared, and the snapshot is restored
- **THEN** all variables declared after the snapshot SHALL be removed from the environment
