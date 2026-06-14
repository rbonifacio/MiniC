## Context

MiniC has assignment statements `id = expr`. The expression parser and identifier parser already exist. The AST has `Stmt::Assign { target: String, value: Box<Expr> }`. This change adds the parser for this statement type only; if/while come in parse-control-flow.

## Goals / Non-Goals

**Goals:**

- Parse `identifier = expression` producing `ir::ast::Stmt::Assign`
- Allow optional whitespace around `=`

**Non-Goals:**

- Parsing if-then-else or while (parse-control-flow)
- Statement sequences or program parsing
- Semantic checks (e.g., target must be declared)

## Decisions

### 1. Module location: `parser/statements.rs`

**Choice:** Create `parser/statements.rs` with an `assignment` parser. Future statement types (if, while) will be added to the same module.

**Rationale:** Keeps all statement parsers in one place; `assignment` is the first of several.

### 2. Grammar: identifier then `=` then expression

**Choice:** Parse identifier, then `=`, then expression. Require identifier (not expression) on the left.

**Rationale:** Matches `Stmt::Assign { target: String, value: Box<Expr> }`; target is always an identifier.

### 3. Whitespace

**Choice:** Use `multispace0` around `=` and before/after identifier and expression.

**Rationale:** Consistent with expression parser; allows `x=1` and `x = 1`.
