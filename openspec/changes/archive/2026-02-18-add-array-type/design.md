## Context

MiniC has scalar values and identifiers. Arrays require: (1) array literals, (2) indexing for read, (3) indexing for write (assignment). The assignment target must support both `x` and `arr[i]`.

## Goals / Non-Goals

**Goals:**

- Parse array literals `[ expr, expr, ... ]`
- Parse index expressions `expr[expr]` (read)
- Parse indexed assignment `arr[i] = expr` (write)
- Support nested indexing `arr[i][j]`

**Non-Goals:**

- Array declaration with size (e.g., `var arr[10]`) — arrays are created by literals or passed as params
- Bounds checking, runtime semantics
- Array length operator

## Decisions

### 1. Array literal: `[ expr, expr, ... ]`

**Choice:** `Expr::ArrayLit(Vec<Expr>)`. Empty array `[]` allowed.

**Rationale:** Familiar syntax. Elements are expressions (literals, identifiers, or nested arrays).

### 2. Index expression: `expr[expr]`

**Choice:** `Expr::Index { base: Box<Expr>, index: Box<Expr> }`. Parsed as postfix on primary.

**Rationale:** `base` can be `Ident`, `Index` (for `arr[i][j]`), or `Call` (for `foo()[0]`). Postfix `[expr]` applies after primary, so `a[i]` parses as primary `a` then postfix `[i]`.

### 3. Assignment target: lvalue

**Choice:** Change `Stmt::Assign { target: String, value }` to `Stmt::Assign { target: Box<Expr>, value }` where `target` is an lvalue: `Expr::Ident` or `Expr::Index`.

**Rationale:** Unifies `x = 1` and `arr[i] = 1`. Lvalue = identifier followed by zero or more `[ expr ]` suffixes. Produces `Expr::Ident("x")` or `Expr::Index { base, index }` (possibly nested).

### 4. Parser order

**Choice:** In `primary`, add array literal before identifier. After `primary`, add postfix for `[ expr ]` (in unary or a new postfix layer). For assignment, parse lvalue (identifier + `[expr]*`) before `=`.

**Rationale:** `[` is unambiguous for array literal. Index is postfix on any primary. Assignment tries lvalue first (identifier then optional `[expr]`), then `=` and value.

### 5. Primary and postfix

**Choice:** Add array literal to primary. Add a postfix step: after parsing primary, repeatedly try `[ expr ]` and wrap in `Expr::Index`. This lives in the expression parser (e.g., extend unary or add `postfix` that wraps primary).

**Rationale:** Standard approach. `a[i][j]` = primary `a`, then `[i]` → Index(a,i), then `[j]` → Index(Index(a,i), j).
