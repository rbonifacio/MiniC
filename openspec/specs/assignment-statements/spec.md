# Assignment Statements

## Purpose

MiniC assignment statement parsing: `identifier = expression` producing `Stmt::Assign`.

## Requirements

### Requirement: Parse assignment statements

The parser SHALL recognize assignment statements of the form `identifier = expression`. The parser SHALL produce `ir::ast::Stmt::Assign` with the identifier as target and the parsed expression as value.

#### Scenario: Simple assignment

- **WHEN** the input is `x = 42` or `count = 0`
- **THEN** the parser SHALL succeed and return `Stmt::Assign { target, value }` with the correct identifier and expression

#### Scenario: Assignment with expression

- **WHEN** the input is `sum = a + b` or `flag = x < 5`
- **THEN** the parser SHALL succeed and return the corresponding `Stmt::Assign` with the full expression as value

#### Scenario: Optional whitespace

- **WHEN** the input is `x=1` or `x = 1` or `x  =  1`
- **THEN** the parser SHALL succeed and return the same `Stmt::Assign`

#### Scenario: Reject invalid assignment

- **WHEN** the input is `= 1` (missing target) or `x` (missing `=` and value) or `1 = x` (invalid target)
- **THEN** the parser SHALL fail (return `Err`)
