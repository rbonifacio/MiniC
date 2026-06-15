## Context

MiniC functions accept a single statement as body (`def foo(x) x = 1`). To support multi-statement functions, we need a way to group statements. The question is: should Block be a new syntactic construct, or a variant of `Stmt`?

## Goals / Non-Goals

**Goals:**

- Add block syntax to group multiple statements
- Enable multi-statement function bodies: `def foo(x, y) { x = x + 1; y = y + 1 }`
- Optionally enable multi-statement if/while bodies

**Non-Goals:**

- Block-scoped variables (no new scope semantics in this change)
- Braces-only blocks without statements (empty block `{}` can be allowed or rejected)

## Decisions

### 1. Block as statement vs. separate syntactic construct

**Choice:** Block is a statement: `Stmt::Block { seq: Vec<Stmt> }`.

**Rationale:** This is the standard approach (C, Java, Rust, Go). A block is a *compound statement*—a statement that contains a sequence of statements. Benefits:

- **Unified grammar**: Function body = statement. If body = statement. While body = statement. A block is just one kind of statement. No special cases.
- **Consistency**: Every place that needs "one or more statements" uses the same rule: accept a statement. A block is one statement.
- **Composability**: Blocks nest naturally: `{ { x = 1 } }`. Both are statements.
- **Simplicity**: No need for `body = block | statement`; body is always `statement`, and block is one alternative in the statement grammar.

**Alternative rejected:** Block as a separate construct (e.g., `function_body = block | statement`) would require special-casing every consumer (function, if, while) and duplicate grammar rules.

### 2. Block syntax: `{ stmt ; stmt ; ... }`

**Choice:** Braces `{` `}` with statements separated by `;`. Trailing `;` optional (or required—pick one for consistency).

**Rationale:** Familiar from C-family languages. Semicolon separates statements; newline alone is ambiguous in MiniC (we don't have significant newlines). Use `separated_list0` or `separated_list1` for the inner sequence.

**Examples:**
- `{ x = 1 }` — one statement
- `{ x = 1; y = 2 }` — two statements
- `{ x = 1; y = 2; z = x + y }` — three statements

### 3. Statement grammar update

**Choice:** Add `block_statement` to the `statement` alt. Order: if | while | call | block | assignment.

**Rationale:** Block must be tried before assignment (otherwise `{` could be parsed as something else). Block is unambiguous: starts with `{`. Placing it before assignment is safe.

### 4. Empty blocks

**Choice:** Allow empty block `{}` (zero statements). `Stmt::Block { seq: vec![] }`.

**Rationale:** Harmless; some languages use it for "do nothing" or placeholder. Can reject later if desired.
