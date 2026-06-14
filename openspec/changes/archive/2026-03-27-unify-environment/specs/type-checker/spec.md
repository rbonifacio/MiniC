## MODIFIED Requirements

### Requirement: Type check entry point
The type checker SHALL accept an unchecked `Program<()>` and return either a typed `Program<Type>` or a single type error. The function signature SHALL be `type_check(program: &UncheckedProgram) -> Result<CheckedProgram, TypeError>` with no registry parameter. The type checker SHALL construct `NativeRegistry::default()` internally and register all native function signatures as `Type::Fun` bindings in the environment before type-checking begins.

#### Scenario: Successful type check
- **WHEN** the input program is well-typed
- **THEN** the type checker SHALL return `Ok(Program<Type>)` with type information attached to each `ExprD` and `StatementD` node

#### Scenario: Type errors
- **WHEN** the input program has a type error
- **THEN** the type checker SHALL return `Err(TypeError)` and SHALL stop at the first error encountered (fail-fast)

---

### Requirement: Function call typing
The type checker SHALL validate function calls by looking up the callee name in the unified `Environment<Type>`. Both user-defined functions (registered as `Type::Fun(params, return_type)` bindings before checking begins) and native stdlib functions (registered from `NativeRegistry::default()` as `Type::Fun` bindings) SHALL be resolved through the same `env.get(name)` lookup. A call is valid when the argument count matches the parameter count and each argument type is compatible with the corresponding parameter type. If a parameter has type `Type::Any`, any argument type SHALL be accepted for that position.

#### Scenario: Call argument count and types
- **WHEN** a function call is type-checked
- **THEN** the argument count SHALL match the parameter count and each argument type SHALL be compatible with the corresponding parameter type (or `Type::Any`)

#### Scenario: Call type mismatch
- **WHEN** argument count or types do not match the registered function signature
- **THEN** the type checker SHALL report a type error and stop

#### Scenario: Stdlib function call with polymorphic parameter
- **WHEN** `print(42)` is type-checked
- **THEN** the call SHALL succeed because `print` is registered with a single `Type::Any` parameter and the argument type `Int` is compatible with `Type::Any`

#### Scenario: Undefined function call
- **WHEN** a call targets a name with no binding in the type environment
- **THEN** the type checker SHALL report a type error identifying the undefined function

---

### Requirement: Function-local scope
The type checker SHALL use function-local scope for function bodies. At the start of checking each function body, the environment SHALL be reset to a clean state containing only function bindings (no variable bindings from previous function checks). Parameters SHALL then be added to this clean environment before the body is checked.

#### Scenario: Parameters in scope
- **WHEN** a function body is type-checked
- **THEN** the function parameters SHALL be in scope for the body

#### Scenario: Isolation between functions
- **WHEN** two functions are type-checked in sequence
- **THEN** variable bindings from the first function SHALL NOT be visible when checking the second function
