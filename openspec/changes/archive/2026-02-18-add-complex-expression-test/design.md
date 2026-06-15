## Context

The expression parser supports relational operators (`>=`), parentheses, multiplication, and addition. A complex expression like `a >= (pi * r * r) + epsilon` exercises multiple features: identifiers, parenthesized multiplication, addition, and relational comparison.

## Goals / Non-Goals

**Goals:**

- Add a test that parses `a >= (pi * r * r) + epsilon` and asserts the AST structure

**Non-Goals:**

- Changing the parser implementation
- Adding new expression operators or syntax

## Decisions

### 1. Test location

**Choice:** Add the test in `tests/parser.rs` with the other expression tests.

**Rationale:** Integration tests live in `tests/parser.rs`; this is an expression test.

### 2. Assertion strategy

**Choice:** Assert that the result is `Expr::Ge` with left `Ident("a")` and right an `Add` whose left is `Mul(Ident("pi"), Mul(Ident("r"), Ident("r")))` and right is `Ident("epsilon")`.

**Rationale:** Validates the full structure; `(pi * r * r)` parses as `pi * (r * r)` due to left-associativity of `*`.
