# Functions

## Purpose

Function declarations and function calls in MiniC.

## Requirements

### Requirement: Parse function declarations

The parser SHALL recognize function declarations in C-style: `return_type identifier ( param_list ) statement`. Return type and parameter types SHALL use lowercase: `int`, `float`, `bool`, `str`, `void` (for Unit). Each parameter SHALL be `type identifier` (type before name). The parser SHALL produce `ir::ast::FunDecl` with name, typed parameter list, return type, and body.

#### Scenario: Function with parameters and return type

- **WHEN** the input is `void foo(int x, int y) x = x + y`
- **THEN** the parser SHALL succeed and return `FunDecl { name: "foo", params: [("x", Int), ("y", Int)], return_type: Unit, body }`

#### Scenario: Function with no parameters

- **WHEN** the input is `void bar() x = 1`
- **THEN** the parser SHALL succeed with `params: []` and return type `void`

#### Scenario: Optional whitespace

- **WHEN** the input is `void  foo  ( int  x , int  y )  x = 1`
- **THEN** the parser SHALL succeed

#### Scenario: Return type required

- **WHEN** a function declaration is parsed
- **THEN** the return type SHALL appear first (C-style)
- **AND** the parser SHALL produce `FunDecl` with a `return_type` field

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
