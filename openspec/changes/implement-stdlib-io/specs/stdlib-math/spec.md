## ADDED Requirements

### Requirement: pow built-in
The stdlib math module SHALL register `pow` in the `NativeRegistry` with signature `(float, float) → float`. `pow` SHALL accept two arguments (`base` and `exp`), coerce `Value::Int` arguments to `f64`, compute `base.powf(exp)` using Rust's `f64::powf`, and return `Value::Float`. The result SHALL follow IEEE 754 semantics (e.g., `pow(2.0, -1.0)` = `0.5`, `pow(0.0, 0.0)` = `1.0`).

#### Scenario: Integer base and exponent
- **WHEN** `pow(2, 10)` is called with `Value::Int` arguments
- **THEN** the result SHALL be `Value::Float(1024.0)`

#### Scenario: Float base and exponent
- **WHEN** `pow(2.0, 0.5)` is called
- **THEN** the result SHALL be `Value::Float` approximately equal to `1.4142135...`

#### Scenario: Negative exponent
- **WHEN** `pow(2.0, -1.0)` is called
- **THEN** the result SHALL be `Value::Float(0.5)`

#### Scenario: Wrong argument count
- **WHEN** `pow(2.0)` is called with only one argument
- **THEN** the result SHALL be `Err(RuntimeError)` describing the arity mismatch

---

### Requirement: sqrt built-in
The stdlib math module SHALL register `sqrt` in the `NativeRegistry` with signature `(float) → float`. `sqrt` SHALL accept one argument, coerce `Value::Int` to `f64`, compute `f64::sqrt`, and return `Value::Float`. The result SHALL follow IEEE 754 semantics (e.g., `sqrt(-1.0)` = `NaN`).

#### Scenario: Perfect square integer
- **WHEN** `sqrt(4)` is called with `Value::Int(4)`
- **THEN** the result SHALL be `Value::Float(2.0)`

#### Scenario: Non-perfect square float
- **WHEN** `sqrt(2.0)` is called
- **THEN** the result SHALL be `Value::Float` approximately equal to `1.4142135...`

#### Scenario: Zero input
- **WHEN** `sqrt(0)` is called
- **THEN** the result SHALL be `Value::Float(0.0)`

#### Scenario: Wrong argument type
- **WHEN** `sqrt(true)` is called with a `Value::Bool`
- **THEN** the result SHALL be `Err(RuntimeError)` describing the type mismatch
