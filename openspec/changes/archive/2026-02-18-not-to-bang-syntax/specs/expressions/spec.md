## MODIFIED Requirements

### Requirement: Boolean expressions

The parser SHALL recognize boolean expressions with operators `and`, `or`, and unary `!`. Precedence SHALL be: `!` (highest), then `and`, then `or` (lowest). `and` and `or` SHALL be left-associative.

#### Scenario: Boolean operators

- **WHEN** the input is `true and false` or `a or b` or `!x`
- **THEN** the parser SHALL succeed and return the corresponding `And`, `Or`, or `Not` expression
