## MODIFIED Requirements

### Requirement: Statement nodes

Add block as a statement variant.

#### Scenario: Block statement

- **WHEN** a block of statements is parsed
- **THEN** it SHALL be represented as `Stmt::Block { seq: Vec<Stmt> }`
- **AND** `seq` SHALL contain the statements in order (zero or more)
