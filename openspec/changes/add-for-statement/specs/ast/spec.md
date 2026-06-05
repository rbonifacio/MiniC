## ADDED Requirements

### Requirement: For statement node

The AST SHALL include a statement node for `for` loops with three header
clauses and a block body. The statement SHALL be parameterized by the type
decoration `Ty` and SHALL use the existing `StatementD<Ty>` and `ExprD<Ty>`
wrappers for its sub-terms.

#### Scenario: For statement structure

- **WHEN** a `for` loop is represented in the AST
- **THEN** it SHALL be `Statement::For { init: Box<StatementD<Ty>>,
  cond: Box<ExprD<Ty>>, update: Box<StatementD<Ty>>, body: Box<StatementD<Ty>> }`

#### Scenario: Init holds declaration or assignment

- **WHEN** a `for` loop is produced by the parser
- **THEN** `init.stmt` SHALL be either `Statement::Decl { … }` or
  `Statement::Assign { … }`.

#### Scenario: Update holds an assignment

- **WHEN** a `for` loop is produced by the parser
- **THEN** `update.stmt` SHALL be `Statement::Assign { … }`.

#### Scenario: Body holds a block

- **WHEN** a `for` loop is produced by the parser
- **THEN** `body.stmt` SHALL be `Statement::Block { … }`.

#### Scenario: Statement recursion

- **WHEN** a `for` loop nests other statements (e.g., another `for` inside its
  body)
- **THEN** those SHALL be represented as nested `StatementD<Ty>` nodes inside
  the `body` block.
