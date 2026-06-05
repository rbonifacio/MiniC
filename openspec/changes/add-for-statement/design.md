## Context

Projeto 2 adds a C-style `for` loop to MiniC. Milestone 1 only covers the
concrete grammar, AST shape, and parser — type checking (scoping of the loop
variable, `cond: bool` requirement) and interpretation belong to Milestone 2.

The existing statement dispatcher in `src/parser/statements.rs` already handles
`if` and `while` with keyword-led `alt` branches and requires block-shaped
bodies. The declaration and assignment parsers each consume a trailing `;`; the
`for` header embeds them between its own `;` separators, so no-semicolon
variants are needed.

## Goals / Non-Goals

**Goals:**

- Parse `for ([init]; [condition]; [update]) block` producing
  `Stmt::For { init, cond, update, body }` where the three header clauses are
  optional.
- Accept both declaration (`int i = 0`) and assignment (`i = 0`) as *init*
  when provided.
- Require *update* to be an assignment when provided.
- Require *body* to be a block (`{ … }`) for parity with `if`/`while`.
- Reject `for` as an identifier.

**Non-Goals:**

- Semantic checks: the type checker will enforce `cond: bool`, the scoping of
  the init variable, and the well-typedness of the update (Milestone 2).
- Interpreter execution semantics (Milestone 2).
- Code generation (Milestone 3).
- Multi-statement init and comma operators.

## Decisions

### 1. Dedicated `Statement::For` variant instead of desugaring to `While`

**Choice:** Extend the AST with a new variant `Statement::For { init, cond,
update, body }` rather than having the parser emit a `Block` wrapping a
`While`.

**Rationale:** The project brief in `docs/09-projects.md` explicitly lists
"Practise adding a new `Statement` variant without breaking existing code" as a
learning objective. A dedicated variant also preserves the original syntactic
shape, which the type checker needs in Milestone 2 to apply tighter scoping
rules to the init variable (it must be visible only inside the loop). Desugaring
in the parser would throw that information away. Desugaring remains a valid
option at interpret/codegen time when convenient.

### 2. Body must be a block

**Choice:** The body of a `for` is required to be a `block_statement` (i.e.
enclosed in `{ … }`), not any statement.

**Rationale:** Matches the existing convention for `if` and `while`, keeps the
language uniform, and avoids the classic dangling-else-style ambiguities when
the body is a bare statement that itself contains `;`.

### 3. Optional `init` / `cond` / `update` with parser-side restrictions

**Choice:** The AST fields `init`, `cond`, and `update` are optional. When
present, `init` is parsed as `Decl`/`Assign` and `update` is parsed as
`Assign`.

**Rationale:** Optional clauses align MiniC with C-style `for` loops while
keeping the parser deterministic. Reusing `StatementD<Ty>` for the present
`init`/`update` cases lets the Milestone 2 type checker dispatch on existing
`Decl`/`Assign` arms without new statement variants.

### 4. No-semicolon init/update helpers

**Choice:** Introduce two helper parsers local to `statements.rs`:
`for_init_clause` (alternates declaration / assignment without `;`) and
`for_update_clause` (assignment without `;`). The outer `for_statement`
consumes the `;` separators itself.

**Rationale:** The existing `decl_statement` and `assignment` parsers consume a
trailing `;`, which cannot be reused inside the `for` header. Duplicating only
the no-semi prefix keeps the behaviour of the normal statement forms
untouched.

### 5. Dispatcher ordering

**Choice:** Place `for_statement` in the `statement` `alt` **before**
`decl_statement` and `assignment` (same tier as `if_statement`/`while_statement`).

**Rationale:** `for` is an unambiguous keyword, so keyword-led alternatives
come first for the same reason `if` and `while` do today — assignment starts
with a plain identifier and would otherwise swallow `for` as a variable name
(were it not also reserved).

## Open Questions

- **Trailing `;` after a `for` block.** The illustrative example in
  `docs/09-projects.md:70` shows `};` after a `for`. Under the current MiniC
  grammar, compound statements (`if`, `while`, blocks) end with `}` and require
  no `;`. Milestone 1 follows that convention; treating the visible `;` as the
  separator of the enclosing block. If the course later decides that simple
  statements always carry `;` even when compound, this can be revisited without
  AST changes.
