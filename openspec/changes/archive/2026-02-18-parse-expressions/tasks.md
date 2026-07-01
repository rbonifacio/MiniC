## 1. Setup

- [x] 1.1 Create `src/parser/expressions.rs` and add `pub mod expressions` to `parser/mod.rs`
- [x] 1.2 Add `From<parser::literals::Literal> for ir::ast::Literal` (or conversion helper) to map parser literals to AST

## 2. Primary Expressions

- [x] 2.1 Implement `primary` parser: literal (mapped to `ir::ast::Literal`), identifier, or parenthesized expression
- [x] 2.2 Add optional whitespace (`multispace0`) between tokens where needed

## 3. Arithmetic Expressions

- [x] 3.1 Implement `unary` parser: optional unary `-` applied to primary
- [x] 3.2 Implement `multiplicative` parser: unary with `*` and `/` (left-associative)
- [x] 3.3 Implement `additive` parser: multiplicative with `+` and `-` (left-associative)

## 4. Relational and Boolean Expressions

- [x] 4.1 Implement `relational` parser: additive with `==`, `!=`, `<`, `<=`, `>`, `>=`
- [x] 4.2 Implement `logical_not` parser: optional `not` applied to relational
- [x] 4.3 Implement `logical_and` parser: logical_not with `and` (left-associative)
- [x] 4.4 Implement `logical_or` parser: logical_and with `or` (left-associative)

## 5. Top-Level and Export

- [x] 5.1 Add `expression` as the top-level parser (entry point)
- [x] 5.2 Export `expression` from `parser/mod.rs`

## 6. Tests

- [x] 6.1 Add unit tests for arithmetic, relational, and boolean expressions
- [x] 6.2 Add tests for precedence (e.g., `1 + 2 * 3`, `x < 5 and y > 0`)
- [x] 6.3 Add tests for invalid input (unbalanced parens, trailing operator)
- [x] 6.4 Run `cargo test` and verify all tests pass
