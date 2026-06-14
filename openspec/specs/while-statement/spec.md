# While Statement

## Purpose

MiniC while statement parsing: `while expression do statement` producing `Stmt::While`.

## Requirements

### Requirement: Parse while statements

The parser SHALL recognize while statements of the form `while expression do statement`. The parser SHALL produce `ir::ast::Stmt::While` with condition and body.

#### Scenario: Simple while

- **WHEN** the input is `while x do y = 1`
- **THEN** the parser SHALL succeed and return `Stmt::While { cond, body }`

#### Scenario: While with compound body

- **WHEN** the input is `while i < 10 do i = i + 1`
- **THEN** the parser SHALL succeed with the full expression and assignment as body

#### Scenario: Nested while

- **WHEN** the input is `while a do while b do x = 1`
- **THEN** the parser SHALL succeed with nested `Stmt::While` nodes

#### Scenario: Optional whitespace

- **WHEN** the input is `while x do y=1` or `while  x  do  y  =  1`
- **THEN** the parser SHALL succeed

#### Scenario: Reject invalid while

- **WHEN** the input is `while x` (missing do/body) or `while do x = 1` (missing condition)
- **THEN** the parser SHALL fail
