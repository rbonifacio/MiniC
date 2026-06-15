## Context

MiniC has arithmetic, relational, and boolean expressions. The parser already has literal and identifier parsers in `parser::literals` and `parser::identifiers`. The parser produces `parser::literals::Literal`; the AST uses `ir::ast::Literal` and `ir::ast::Expr`. The expression parser must compose atoms into a precedence-correct expression tree and produce `ir::ast::Expr`.

## Goals / Non-Goals

**Goals:**

- Implement expression parser in `parser/expressions.rs` producing `ir::ast::Expr`
- Compose literals and identifiers as atoms; map to `ir::ast` types
- Use precedence climbing or recursive descent for correct precedence
- Support parentheses for grouping

**Non-Goals:**

- Parsing statements (assignments, if, while)
- Whitespace-insensitive parsing (optional; can use `multispace0` between tokens)
- Operator precedence table as a separate data structure

## Decisions

### 1. Recursive descent with precedence levels

**Choice:** Use a recursive-descent style with separate parser functions per precedence level: `primary` → `unary` → `multiplicative` → `additive` → `relational` → `logical_or` → `logical_and` → `logical_not` (or combine and/or in one level).

**Rationale:** Precedence is arithmetic > relational > boolean. Within arithmetic: unary `-` > `*`/`/` > `+`/`-`. Recursive descent maps each level to a function; lower-precedence calls higher-precedence.

**Alternative considered:** Precedence climbing (single function with operator table). Rejected; recursive descent is more explicit and easier to debug for a small set of operators.

### 2. Map parser literals to `ir::ast::Literal` at parse time

**Choice:** When parsing a literal as a primary expression, convert `parser::literals::Literal` to `ir::ast::Literal` immediately via a small helper or `From` impl.

**Rationale:** The expression parser returns `ir::ast::Expr`; it must produce `ir::ast::Literal` for `Expr::Literal`. The parser's `Literal` enum has the same structure as `ir::ast::Literal` and can be converted with a simple match.

**Alternative considered:** Change parser literals to return `ir::ast::Literal` directly. Rejected; would require modifying the parse-literals-and-identifiers change; conversion at the expression boundary is simpler.

### 3. Whitespace between tokens

**Choice:** Use `nom::character::complete::multispace0` (or similar) between operators and operands. Optional whitespace is allowed around binary operators.

**Rationale:** MiniC is likely to allow `1+2` and `1 + 2`. Nom's `multispace0` consumes zero or more spaces/tabs/newlines without requiring them.

### 4. Left-associative binary operators

**Choice:** Use `fold_left` or `many0` + `fold` pattern so that `a + b + c` parses as `(a + b) + c`.

**Rationale:** `+`, `-`, `*`, `/`, `and`, `or` are left-associative. `fold_left` naturally produces left-associative trees.

### 5. Module layout

**Choice:** Single file `parser/expressions.rs` with `primary`, `unary`, `multiplicative`, `additive`, `relational`, `logical_or`, `logical_and` (or combined logical), and `expression` as the top-level entry point.

**Rationale:** Expression parsing is cohesive; one file keeps it manageable. Export `expression` from `parser/mod.rs`.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Precedence bugs (e.g., `and` before `or`) | Add tests for each precedence scenario from the spec |
| Unary `not` vs `-` in same expression | Define order: `not` applies to boolean; `-` to arithmetic; they don't mix in primary |
| Left recursion in grammar | Use recursive descent: each level calls the next higher level, never itself directly for the same construct |
