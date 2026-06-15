# Expressions

## Purpose

MiniC expression parsing: arithmetic, relational, and boolean expressions producing AST nodes.

## Requirements

### Requirement: Expression parser produces AST nodes

The expression parser SHALL produce `ir::ast::Expr` values. It SHALL use the existing literal and identifier parsers as atoms and compose them into expression trees.

#### Scenario: Parser returns Expr type

- **WHEN** the expression parser successfully parses input
- **THEN** it SHALL return `IResult<&str, ir::ast::Expr>`

#### Scenario: Literals map to AST

- **WHEN** a literal (integer, float, string, boolean) is parsed as an expression
- **THEN** it SHALL be wrapped as `Expr::Literal(ir::ast::Literal::...)`

#### Scenario: Identifiers map to AST

- **WHEN** an identifier is parsed as an expression
- **THEN** it SHALL be represented as `Expr::Ident(String)`

#### Scenario: Complex expression with parentheses and operators

- **WHEN** the input is `a >= (pi * r * r) + epsilon`
- **THEN** the parser SHALL succeed and return an `Expr::Ge` with:
  - Left: `Expr::Ident("a")`
  - Right: `Expr::Add` of `(pi * r * r)` and `epsilon` (parenthesized multiplication, then addition)

---

### Requirement: Arithmetic expressions

The parser SHALL recognize arithmetic expressions with operators `+`, `-`, `*`, `/` and unary `-`. Precedence SHALL be: unary `-` (highest), then `*`/`/`, then `+`/`-` (lowest). `+` and `-` SHALL be left-associative; `*` and `/` SHALL be left-associative.

#### Scenario: Binary arithmetic

- **WHEN** the input is `1 + 2` or `10 - 3` or `4 * 5` or `20 / 4`
- **THEN** the parser SHALL succeed and return the corresponding `Add`, `Sub`, `Mul`, or `Div` expression

#### Scenario: Unary minus

- **WHEN** the input is `-x` or `-(1 + 2)`
- **THEN** the parser SHALL succeed and return `Expr::Neg(...)`

#### Scenario: Precedence

- **WHEN** the input is `1 + 2 * 3`
- **THEN** the parser SHALL produce a tree equivalent to `1 + (2 * 3)` (multiplication binds tighter)

#### Scenario: Parentheses override precedence

- **WHEN** the input is `(1 + 2) * 3`
- **THEN** the parser SHALL produce a tree equivalent to `(1 + 2) * 3`

---

### Requirement: Relational expressions

The parser SHALL recognize relational expressions with operators `==`, `!=`, `<`, `<=`, `>`, `>=`. Relational operators SHALL have lower precedence than arithmetic.

#### Scenario: Relational operators

- **WHEN** the input is `a == b` or `x != y` or `i < 10` or `j >= 0`
- **THEN** the parser SHALL succeed and return the corresponding `Eq`, `Ne`, `Lt`, `Le`, `Gt`, or `Ge` expression

#### Scenario: Relational vs arithmetic precedence

- **WHEN** the input is `1 + 2 < 5`
- **THEN** the parser SHALL produce a tree equivalent to `(1 + 2) < 5` (arithmetic binds tighter than relational)

---

### Requirement: Boolean expressions

The parser SHALL recognize boolean expressions with operators `and`, `or`, and unary `!`. Precedence SHALL be: `!` (highest), then `and`, then `or` (lowest). `and` and `or` SHALL be left-associative.

#### Scenario: Boolean operators

- **WHEN** the input is `true and false` or `a or b` or `!x`
- **THEN** the parser SHALL succeed and return the corresponding `And`, `Or`, or `Not` expression

#### Scenario: Boolean vs relational precedence

- **WHEN** the input is `x < 5 and y > 0`
- **THEN** the parser SHALL produce a tree equivalent to `(x < 5) and (y > 0)` (relational binds tighter than boolean)

---

### Requirement: Primary expressions

The parser SHALL recognize primary expressions: literals, identifiers, and parenthesized expressions.

#### Scenario: Literal as primary

- **WHEN** the input is `42` or `3.14` or `"hi"` or `true`
- **THEN** the parser SHALL succeed and return the corresponding literal expression

#### Scenario: Identifier as primary

- **WHEN** the input is `x` or `count`
- **THEN** the parser SHALL succeed and return `Expr::Ident(...)`

#### Scenario: Parenthesized expression

- **WHEN** the input is `(1 + 2)` or `(x)`
- **THEN** the parser SHALL succeed and return the inner expression

---

### Requirement: Reject invalid input

The parser SHALL fail (return `Err`) when the input is not a valid expression.

#### Scenario: Invalid operator

- **WHEN** the input is `1 +` or `* 2` or empty
- **THEN** the parser SHALL fail

#### Scenario: Unbalanced parentheses

- **WHEN** the input is `(1 + 2` or `1 + 2)`
- **THEN** the parser SHALL fail
