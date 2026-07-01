## MODIFIED Requirements

### Requirement: Expression parser produces AST nodes

#### Scenario: Complex expression with parentheses and operators

- **WHEN** the input is `a >= (pi * r * r) + epsilon`
- **THEN** the parser SHALL succeed and return an `Expr::Ge` with:
  - Left: `Expr::Ident("a")`
  - Right: `Expr::Add` of `(pi * r * r)` and `epsilon` (parenthesized multiplication, then addition)
