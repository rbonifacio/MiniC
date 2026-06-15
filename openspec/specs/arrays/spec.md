# Arrays

## Purpose

MiniC array literals, index expressions, and indexed assignment parsing.

## Requirements

### Requirement: Parse array literals

The parser SHALL recognize array literals of the form `[ expr, expr, ... ]`. The parser SHALL produce `Expr::ArrayLit(elements)`.

#### Scenario: Array with elements

- **WHEN** the input is `[1, 2, 3]` or `[a, b, c]`
- **THEN** the parser SHALL succeed and return `Expr::ArrayLit` with the correct elements

#### Scenario: Empty array

- **WHEN** the input is `[]`
- **THEN** the parser SHALL succeed with `Expr::ArrayLit(vec![])`

#### Scenario: Array in expression

- **WHEN** the input is `[1, 2][0]` or `a + [1, 2, 3][i]`
- **THEN** the parser SHALL succeed with the array and index as subexpressions

---

### Requirement: Parse index expressions

The parser SHALL recognize index expressions of the form `expr[expr]`. The parser SHALL produce `Expr::Index { base, index }`.

#### Scenario: Simple index

- **WHEN** the input is `arr[i]` or `x[0]`
- **THEN** the parser SHALL succeed with `base` and `index` as the corresponding expressions

#### Scenario: Nested index

- **WHEN** the input is `arr[i][j]`
- **THEN** the parser SHALL succeed with nested `Expr::Index`

---

### Requirement: Parse indexed assignment

The parser SHALL recognize assignments to indexed expressions: `identifier [ expr ] [ expr ] ... = expr`. The parser SHALL produce `Stmt::Assign { target, value }` where target is `Expr::Index` (possibly nested).

#### Scenario: Indexed assignment

- **WHEN** the input is `arr[i] = 1` or `matrix[i][j] = x`
- **THEN** the parser SHALL succeed with target as the appropriate Index expression
