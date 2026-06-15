## ADDED Requirements

### Requirement: Literal evaluation
The interpreter SHALL evaluate every `Literal` variant in the AST to the corresponding `Value` variant with the same data.

#### Scenario: Integer literal
- **WHEN** `Expr::Literal(Literal::Int(7))` is evaluated
- **THEN** the result SHALL be `Value::Int(7)`

#### Scenario: Float literal
- **WHEN** `Expr::Literal(Literal::Float(3.14))` is evaluated
- **THEN** the result SHALL be `Value::Float(3.14)`

#### Scenario: Boolean literal true
- **WHEN** `Expr::Literal(Literal::Bool(true))` is evaluated
- **THEN** the result SHALL be `Value::Bool(true)`

#### Scenario: String literal
- **WHEN** `Expr::Literal(Literal::Str("hello"))` is evaluated
- **THEN** the result SHALL be `Value::Str("hello".to_string())`

---

### Requirement: Identifier evaluation
The interpreter SHALL evaluate `Expr::Ident(name)` by looking up `name` in the current `RuntimeEnv` and returning its bound `Value`. If the name is not found the interpreter SHALL return a `RuntimeError`.

#### Scenario: Declared variable lookup
- **WHEN** variable `x` is bound to `Value::Int(5)` in the environment and `Expr::Ident("x")` is evaluated
- **THEN** the result SHALL be `Value::Int(5)`

---

### Requirement: Arithmetic expression evaluation
The interpreter SHALL evaluate binary arithmetic expressions (`Add`, `Sub`, `Mul`, `Div`) according to the following rules:
- `Int op Int` → `Value::Int`
- `Float op Float` → `Value::Float`
- `Int op Float` or `Float op Int` → `Value::Float` (int is coerced to float)
Unary negation (`Neg`) SHALL negate the numeric value and preserve its variant.

#### Scenario: Integer addition
- **WHEN** `2 + 3` is evaluated with both operands as `Value::Int`
- **THEN** the result SHALL be `Value::Int(5)`

#### Scenario: Mixed int/float addition
- **WHEN** `2 + 1.5` is evaluated with operands `Value::Int(2)` and `Value::Float(1.5)`
- **THEN** the result SHALL be `Value::Float(3.5)`

#### Scenario: Unary negation of integer
- **WHEN** `-4` is evaluated
- **THEN** the result SHALL be `Value::Int(-4)`

---

### Requirement: Comparison expression evaluation
The interpreter SHALL evaluate relational expressions (`Lt`, `Le`, `Gt`, `Ge`) on numeric operands and return `Value::Bool`. Mixed `Int`/`Float` comparisons SHALL coerce the integer operand to float before comparing.

#### Scenario: Less-than on integers
- **WHEN** `3 < 5` is evaluated
- **THEN** the result SHALL be `Value::Bool(true)`

#### Scenario: Greater-than-or-equal with mixed types
- **WHEN** `3.0 >= 2` is evaluated
- **THEN** the result SHALL be `Value::Bool(true)`

---

### Requirement: Equality expression evaluation
The interpreter SHALL evaluate `Eq` and `Ne` on any two values of compatible types and return `Value::Bool`. An `Int` and `Float` with the same numeric value SHALL be considered equal.

#### Scenario: Integer equality true
- **WHEN** `4 == 4` is evaluated
- **THEN** the result SHALL be `Value::Bool(true)`

#### Scenario: Boolean inequality
- **WHEN** `true != false` is evaluated
- **THEN** the result SHALL be `Value::Bool(true)`

---

### Requirement: Logical expression evaluation
The interpreter SHALL evaluate `Not`, `And`, and `Or` on `Value::Bool` operands. `And` and `Or` SHALL short-circuit: `And` SHALL NOT evaluate the right operand if the left is `false`; `Or` SHALL NOT evaluate the right operand if the left is `true`.

#### Scenario: Logical not
- **WHEN** `!true` is evaluated
- **THEN** the result SHALL be `Value::Bool(false)`

#### Scenario: Short-circuit and
- **WHEN** `false and <expr>` is evaluated and `<expr>` would produce a runtime error if evaluated
- **THEN** the result SHALL be `Value::Bool(false)` without evaluating `<expr>`

#### Scenario: Short-circuit or
- **WHEN** `true or <expr>` is evaluated and `<expr>` would produce a runtime error if evaluated
- **THEN** the result SHALL be `Value::Bool(true)` without evaluating `<expr>`

---

### Requirement: Array literal evaluation
The interpreter SHALL evaluate `Expr::ArrayLit(elements)` by evaluating each element in order and collecting the results into `Value::Array(Vec<Value>)`.

#### Scenario: Array literal with three elements
- **WHEN** `[10, 20, 30]` is evaluated
- **THEN** the result SHALL be `Value::Array([Value::Int(10), Value::Int(20), Value::Int(30)])`

---

### Requirement: Array index evaluation
The interpreter SHALL evaluate `Expr::Index { base, index }` by evaluating `base` to a `Value::Array`, evaluating `index` to a `Value::Int`, and returning the element at that index. If the index is out of bounds the interpreter SHALL return a `RuntimeError`.

#### Scenario: Valid array index
- **WHEN** `arr[1]` is evaluated and `arr` is `Value::Array([Value::Int(10), Value::Int(20)])`
- **THEN** the result SHALL be `Value::Int(20)`

#### Scenario: Out-of-bounds index
- **WHEN** `arr[5]` is evaluated and `arr` has only 2 elements
- **THEN** the interpreter SHALL return `Err(RuntimeError)`
