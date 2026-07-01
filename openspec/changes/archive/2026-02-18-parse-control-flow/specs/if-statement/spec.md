## ADDED Requirements

### Requirement: Parse if-then-else statements

The parser SHALL recognize if-then-else statements of the form `if expression then statement [else statement]`. The parser SHALL produce `ir::ast::Stmt::If` with condition, then-branch, and optional else-branch.

#### Scenario: If without else

- **WHEN** the input is `if x then y = 1`
- **THEN** the parser SHALL succeed and return `Stmt::If { cond, then_branch, else_branch: None }`

#### Scenario: If with else

- **WHEN** the input is `if x then y = 1 else y = 0`
- **THEN** the parser SHALL succeed and return `Stmt::If` with `else_branch: Some(...)`

#### Scenario: Nested if

- **WHEN** the input is `if a then if b then x = 1 else x = 2`
- **THEN** the parser SHALL succeed with nested `Stmt::If` nodes

#### Scenario: Optional whitespace

- **WHEN** the input is `if x then y=1` or `if  x  then  y  =  1`
- **THEN** the parser SHALL succeed

#### Scenario: Reject invalid if

- **WHEN** the input is `if x` (missing then/body) or `if then x = 1` (missing condition)
- **THEN** the parser SHALL fail
