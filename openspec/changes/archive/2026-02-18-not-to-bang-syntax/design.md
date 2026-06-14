## Context

The expression parser currently uses the keyword `not` for logical negation (e.g., `not x`). The user wants to switch to the C-style `!` operator.

## Goals / Non-Goals

**Goals:**

- Parse `!expr` instead of `not expr` for logical negation
- AST remains `Expr::Not(Box<Expr>)`; only concrete syntax changes

**Non-Goals:**

- Changing `and` or `or` to `&&` or `||` (future change if desired)
- Adding `not` as an alias for `!` (no backward compatibility)

## Decisions

### 1. Parser change: `tag("not")` → `char('!')`

**Choice:** In `logical_not`, replace `preceded(multispace0, tag("not"))` with `preceded(multispace0, char('!'))`.

**Rationale:** `!` is a single character; `char('!')` is the standard nom combinator. Optional whitespace before `!` and after it (before the operand) remains via `multispace0`.

### 2. No space required after `!`

**Choice:** Allow `!x` and `! x`; `multispace0` handles both.

**Rationale:** Consistent with other unary operators (e.g., `-x`); C-style allows `!x`.
