## ADDED Requirements

### Requirement: Parse function declarations

The parser SHALL recognize function declarations of the form `def identifier ( param_list ) statement`. The parser SHALL produce `ir::ast::FunDecl` with name, parameter list, and body.

#### Scenario: Function with parameters

- **WHEN** the input is `def foo(x, y) x = x + y`
- **THEN** the parser SHALL succeed and return `FunDecl { name: "foo", params: ["x", "y"], body }`

#### Scenario: Function with no parameters

- **WHEN** the input is `def bar() x = 1`
- **THEN** the parser SHALL succeed with `params: []`

#### Scenario: Optional whitespace

- **WHEN** the input is `def  foo  ( x , y )  x = 1`
- **THEN** the parser SHALL succeed

---

### Requirement: Parse function calls as expressions

The parser SHALL recognize function calls of the form `identifier ( expr_list )` in expression context. The parser SHALL produce `Expr::Call { name, args }`.

#### Scenario: Call with arguments

- **WHEN** the input is `foo(1, 2)` or `bar(a + b, x)`
- **THEN** the parser SHALL succeed and return `Expr::Call` with the correct name and argument expressions

#### Scenario: Call with no arguments

- **WHEN** the input is `baz()`
- **THEN** the parser SHALL succeed with `args: []`

#### Scenario: Call in larger expression

- **WHEN** the input is `foo(1) + 2` or `if bar() then x = 1`
- **THEN** the parser SHALL succeed with the call as a subexpression

---

### Requirement: Parse function calls as statements

The parser SHALL recognize standalone function calls of the form `identifier ( expr_list )` as statements. The parser SHALL produce `Stmt::Call { name, args }`.

#### Scenario: Call statement

- **WHEN** the input is `foo(1, 2)` at statement level
- **THEN** the parser SHALL succeed and return `Stmt::Call { name, args }`
