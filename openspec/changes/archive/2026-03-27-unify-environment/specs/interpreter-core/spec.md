## MODIFIED Requirements

### Requirement: Interpret entry point
The interpreter SHALL expose a public `interpret(program: &CheckedProgram) -> Result<(), RuntimeError>` function with no registry parameter. Internally it SHALL construct `NativeRegistry::default()`, build an `Environment<Value>`, register all user-defined functions as `Value::Fn(FnValue::UserDefined(...))` bindings, register all native stdlib functions as `Value::Fn(FnValue::Native(...))` bindings, locate the `main` function entry in the environment, execute it, and return `Ok(())` on success or `Err(RuntimeError)` on failure.

#### Scenario: Successful execution of a minimal program
- **WHEN** `interpret` is called with a `CheckedProgram` containing a `void main()` function with an empty body
- **THEN** the function SHALL return `Ok(())`

#### Scenario: Missing main function
- **WHEN** `interpret` is called with a `CheckedProgram` that contains no function named `main`
- **THEN** the function SHALL return `Err(RuntimeError)` describing the missing entry point

---

### Requirement: Value representation
The interpreter SHALL represent all MiniC runtime values with a `Value` enum: `Int(i64)`, `Float(f64)`, `Bool(bool)`, `Str(String)`, `Array(Vec<Value>)`, `Void`, and `Fn(FnValue)`. Every operation that produces a computable value SHALL return one of the scalar or composite variants. `Value::Fn` is an internal representation used for environment storage and dispatch; it SHALL NOT be produced by evaluating a MiniC expression directly.

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

### Requirement: Runtime environment
The interpreter SHALL use `Environment<Value>` (the unified parametric environment) to store both variable bindings and function bindings. There SHALL be no separate `RuntimeEnv` struct. The environment SHALL support snapshot/restore for function call scoping and `names()`/`remove_new()` for block scoping.

#### Scenario: Variable binding is visible within its scope
- **WHEN** a variable `x` is declared inside a block `{ int x = 1; }`
- **THEN** `x` SHALL be accessible within that block and SHALL NOT be accessible after the block exits

#### Scenario: Function bindings persist across scoping operations
- **WHEN** block scoping via `remove_new` is applied
- **THEN** function bindings (registered before any block) SHALL remain in the environment because they are present in the outer name set
