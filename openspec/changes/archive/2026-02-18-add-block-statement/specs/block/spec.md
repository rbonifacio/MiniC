## ADDED Requirements

### Requirement: Parse block statements

The parser SHALL recognize blocks of the form `{ stmt ; stmt ; ... }`. The parser SHALL produce `Stmt::Block { seq }` with the parsed statements.

#### Scenario: Block with one statement

- **WHEN** the input is `{ x = 1 }`
- **THEN** the parser SHALL succeed and return `Stmt::Block { seq }` with `seq.len() == 1`

#### Scenario: Block with multiple statements

- **WHEN** the input is `{ x = 1; y = 2 }` or `{ x = 1; y = 2; z = x + y }`
- **THEN** the parser SHALL succeed with `seq` containing the statements in order

#### Scenario: Empty block

- **WHEN** the input is `{}`
- **THEN** the parser SHALL succeed with `Stmt::Block { seq: vec![] }`

#### Scenario: Block in function body

- **WHEN** the input is `def foo(x, y) { x = x + 1; y = y + 1 }`
- **THEN** the parser SHALL succeed and the function body SHALL be `Stmt::Block` with two statements

#### Scenario: Block in if/while body

- **WHEN** the input is `if x then { a = 1; b = 2 }` or `while x do { a = 1; b = 2 }`
- **THEN** the parser SHALL succeed with the body as `Stmt::Block`
