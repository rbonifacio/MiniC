# Type Checker

## Purpose

Semantic analysis for MiniC: type-check the unchecked AST (`Program<()>`), report type errors, and produce a typed AST (`Program<Type>`). Supports int/float coercion, function return type validation, and function-local scope. Fails at the first error.

## Requirements

### Requirement: Type check entry point

The type checker SHALL accept an unchecked `Program<()>` and return either a typed `Program<Type>` or a single type error. The function signature SHALL be `type_check(program: &UncheckedProgram) -> Result<CheckedProgram, TypeError>` with no registry parameter. The type checker SHALL construct `NativeRegistry::default()` internally and register all native function signatures as `Type::Fun` bindings in the environment before type-checking begins.

#### Scenario: Successful type check

- **WHEN** the input program is well-typed
- **THEN** the type checker SHALL return `Ok(Program<Type>)` with type information attached to each `ExprD` and `StatementD` node

#### Scenario: Type errors

- **WHEN** the input program has a type error
- **THEN** the type checker SHALL return `Err(TypeError)` and SHALL stop at the first error encountered (fail-fast)

---

### Requirement: Literal and identifier typing

The type checker SHALL assign types to literals and identifiers from context.

#### Scenario: Literal types

- **WHEN** a literal is type-checked
- **THEN** `Int` literal SHALL have type `Int`, `Float` → `Float`, `Str` → `Str`, `Bool` → `Bool`

#### Scenario: Identifier types

- **WHEN** an identifier is type-checked
- **THEN** its type SHALL be looked up from the current scope (function-local or global)

#### Scenario: Undeclared identifier

- **WHEN** an identifier is used before being assigned (or not in scope)
- **THEN** the type checker SHALL report a type error and stop

---

### Requirement: Main function required

The type checker SHALL require a `main` function as the program entry point.

#### Scenario: Main exists

- **WHEN** the program has a function named `main`
- **THEN** the type checker SHALL proceed with type-checking

#### Scenario: Main missing

- **WHEN** the program has no function named `main`
- **THEN** the type checker SHALL report a type error and stop

---

### Requirement: Function-local scope

The type checker SHALL use function-local scope for function bodies. At the start of checking each function body, the environment SHALL be reset to a clean state containing only function bindings (no variable bindings from previous function checks). Parameters SHALL then be added to this clean environment before the body is checked.

#### Scenario: Parameters in scope

- **WHEN** a function body is type-checked
- **THEN** the function parameters SHALL be in scope for the body

#### Scenario: Main body scope

- **WHEN** the `main` function body is type-checked
- **THEN** variables SHALL use local scope (assigned before use, or from params)

#### Scenario: Isolation between functions

- **WHEN** two functions are type-checked in sequence
- **THEN** variable bindings from the first function SHALL NOT be visible when checking the second function

---

### Requirement: Function return type validation

The type checker SHALL validate that function bodies conform to their declared return type.

#### Scenario: Return type match

- **WHEN** a function has a declared return type and its body is type-checked
- **THEN** the body's type SHALL match the declared return type (e.g., last expression or statement type)

#### Scenario: Return type mismatch

- **WHEN** the body type does not match the declared return type
- **THEN** the type checker SHALL report a type error and stop

---

### Requirement: Int/float coercion

The type checker SHALL promote mixed int/float operands to float for arithmetic and relational operators.

#### Scenario: Int and int

- **WHEN** both operands of `+`, `-`, `*`, or `/` are `Int`
- **THEN** the result type SHALL be `Int`

#### Scenario: Int and float (either order)

- **WHEN** one operand is `Int` and the other is `Float` for `+`, `-`, `*`, or `/`
- **THEN** the result type SHALL be `Float`

#### Scenario: Float and float

- **WHEN** both operands of `+`, `-`, `*`, or `/` are `Float`
- **THEN** the result type SHALL be `Float`

#### Scenario: Relational operators

- **WHEN** operands of `==`, `!=`, `<`, `<=`, `>`, `>=` are numeric (int or float)
- **THEN** coercion SHALL apply and the result type SHALL be `Bool`

---

### Requirement: Boolean and string typing

The type checker SHALL enforce types for boolean and string operations.

#### Scenario: Boolean operators

- **WHEN** operands of `and`, `or`, or `!` are type-checked
- **THEN** operands SHALL have type `Bool` and the result SHALL be `Bool`; otherwise report error and stop

#### Scenario: String operations

- **WHEN** `+` is applied to operands
- **THEN** the type checker SHALL accept `Str + Str` → `Str`; mixed with numeric SHALL be an error

---

### Requirement: Array typing

The type checker SHALL type-check array literals and index expressions.

#### Scenario: Array literal

- **WHEN** an array literal `[e1, e2, ...]` is type-checked
- **THEN** all elements SHALL have the same type (after coercion) and the result SHALL be `Array(elem_ty)`

#### Scenario: Index expression

- **WHEN** an index expression `base[i]` is type-checked
- **THEN** `base` SHALL have type `Array(t)`, `i` SHALL have type `Int`, and the result SHALL be `t`

#### Scenario: Indexed assignment

- **WHEN** an assignment target is an index expression
- **THEN** the target type SHALL match the value type (same rules as simple assignment)

---

### Requirement: Assignment typing

The type checker SHALL ensure assignment target and value types are compatible.

#### Scenario: Simple assignment

- **WHEN** `x = expr` is type-checked
- **THEN** the target (identifier or index) and value SHALL have compatible types; the identifier SHALL be updated in scope with the value type

#### Scenario: Type mismatch

- **WHEN** assignment target and value have incompatible types (e.g., `Bool` and `Int`)
- **THEN** the type checker SHALL report a type error and stop

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
