## Why

Statements (assignments, if-then-else, while) require parsing expressions for right-hand sides and conditions. The expression parser composes literals and identifiers into arithmetic, relational, and boolean expressions with correct precedence. This change establishes the expression layer that all statement parsing depends on.

## What Changes

- Add expression parser in `parser/expressions.rs` that produces `ir::ast::Expr`
- Compose existing literal and identifier parsers as expression atoms
- Implement precedence: arithmetic (highest) → relational → boolean (lowest)
- Arithmetic: unary `-`, binary `+`, `-`, `*`, `/`
- Relational: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Boolean: unary `not`, binary `and`, `or`
- Support parenthesized expressions for explicit grouping

## Capabilities

### New Capabilities

- `expressions`: Parsing of arithmetic, relational, and boolean expressions in MiniC, producing `ir::ast::Expr` nodes with correct precedence and associativity.

### Modified Capabilities

- (none)

## Impact

- **New module**: `src/parser/expressions.rs`
- **Dependencies**: Uses `parser::literals`, `parser::identifiers`, and `ir::ast`
- **Integration**: Parser literals may need to map to `ir::ast::Literal`; expression parser produces `ir::ast::Expr`
