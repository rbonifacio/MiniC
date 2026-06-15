## MODIFIED Requirements

### Requirement: Expression nodes

Add array literal and index as expression variants.

#### Scenario: Array literal

- **WHEN** an array literal is parsed
- **THEN** it SHALL be represented as `Expr::ArrayLit(Vec<Expr>)`
- **AND** the vector SHALL contain the element expressions in order (zero or more)

#### Scenario: Index expression

- **WHEN** an index expression is parsed (e.g., `arr[i]`)
- **THEN** it SHALL be represented as `Expr::Index { base: Box<Expr>, index: Box<Expr> }`
- **AND** `base` SHALL be the indexed expression and `index` SHALL be the index expression

### Requirement: Statement nodes (Assign)

The assignment target SHALL support both simple identifiers and indexed expressions.

#### Scenario: Assign with lvalue target

- **WHEN** an assignment is parsed
- **THEN** it SHALL be represented as `Stmt::Assign { target: Box<Expr>, value: Box<Expr> }`
- **AND** `target` SHALL be an lvalue: `Expr::Ident` or `Expr::Index` (possibly nested)
