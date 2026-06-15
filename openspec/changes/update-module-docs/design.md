## Context

MiniC is used as a teaching tool. Its six subsystems — `ir`, `parser`,
`semantic`, `environment`, `interpreter`, and `stdlib` — are deliberately
small so students can read and understand them in full. The current doc
comments are either missing or one-liners that restate the obvious. Students
reading the code for the first time have no guidance on *why* the code is
structured the way it is, which Rust constructs are doing what job, or how
the pieces connect.

The primary entry point for documentation is each module's `mod.rs` (or the
file that *is* the module). Sub-module files (`eval_expr.rs`, `exec_stmt.rs`,
`env.rs`, `math.rs`, `io.rs`) also carry design-specific decisions that are
best explained where the code lives.

## Goals / Non-Goals

**Goals:**
- Every module has a `//!` top-level comment with: purpose, public
  API surface, and design rationale.
- Design decisions are explained in plain English. Where a Rust-specific
  construct is used (enum, generic, function pointer, trait impl), a brief
  lay explanation is included the *first* time it appears.
- The student can read the doc comment of any single module and understand
  that module's role without having to read the others first.
- Comments stay accurate. They describe observable behaviour in the current
  code, not aspirational future design.

**Non-Goals:**
- Per-function or per-field `///` doc comments (out of scope; the focus
  is the module narrative, not API reference).
- HTML/rustdoc cross-linking between modules (nice to have, deferred).
- Changing any source code logic, types, function signatures, or tests.

## Decisions

### D1 — `//!` inner doc comments, not README files

**Choice:** Use Rust's `//!` module-level comments directly in source files.

**Rationale:** `cargo doc` renders these as the module's documentation page,
keeping docs co-located with code and reducing the risk of drift. An external
README would need to be manually kept in sync and isn't navigable from an IDE.

**Alternative considered:** A `docs/` folder with Markdown files per module.
Rejected because it breaks the "docs live next to code" principle and requires
students to context-switch between two locations.

### D2 — Two-part structure per module comment

**Choice:** Every module comment has exactly two named sections:
`# Overview` and `# Design Decisions`.

**Rationale:** Separating *what* (overview) from *why* (design decisions)
mirrors how good technical writing works. Students looking for orientation
read the overview; students asking "why isn't this done differently?" go to
design decisions.

**Alternative considered:** Free-form prose. Rejected because without a
consistent structure, docs across six modules would diverge in style and
depth, making the codebase harder to navigate as a learning resource.

### D3 — Design decisions placed at the module that owns them

**Choice:** Design decisions live in the module where the relevant code
is defined, not in a single top-level document.

**Rationale:** Students typically start by reading a specific module.
Centralising design rationale would require them to jump to a separate file.
Putting rationale next to the code means reading `interpreter/mod.rs`
explains why the tree-walking approach was chosen; reading `environment/env.rs`
explains `snapshot`/`restore`; and so on.

**Alternative considered:** A single `ARCHITECTURE.md` covering all decisions.
Rejected because it disconnects rationale from the code it explains and
becomes stale faster.

### D4 — Rust constructs explained on first appearance per module

**Choice:** When a module uses a Rust-specific pattern for the first time
(e.g., `enum` with data, a generic `<V>` type parameter, `fn` function
pointer types), a short plain-English gloss is included in that module's doc.

**Rationale:** The audience has programming fundamentals but not Rust
expertise. A doc that uses the word "trait" or "enum variant" without
explanation will lose a significant portion of readers at the first hurdle.

**Alternative considered:** Link to the Rust Book. Rejected because external
links rot and interrupt reading flow. A one-sentence gloss is faster to read
and stays valid as long as the code does.

### D5 — Design decisions per module

The table below records which design decision belongs to which module's docs.
This is the authoritative guide for the implementation phase.

| Module | Design decisions to document |
|---|---|
| `ir` | Why a separate IR/AST phase exists; the `Checked*` vs unchecked AST split; why AST nodes are plain data (no methods) |
| `parser` | nom parser combinators vs hand-written recursive descent; sub-module decomposition by syntactic category; why the parser produces an untyped AST |
| `semantic` | Why type checking is a separate pass (not integrated into parsing); use of `Environment<Type>` for variable tracking; the `CheckedProgram` output type |
| `environment` | The single parametric `Environment<V>` serving both type checker and interpreter; `snapshot`/`restore` for function call scoping; `names`/`remove_new` for block scoping |
| `interpreter` | Tree-walking approach; `eval_expr`/`exec_stmt` decomposition; the `Value` enum as a runtime type; `FnValue` unifying user-defined and native functions; why functions are stored in the same environment as variables |
| `stdlib` | `NativeRegistry` pattern; `NativeEntry` bundling type signature with implementation; why `print` uses `Type::Any` |

## Risks / Trade-offs

- [Risk] Doc comments become stale as the code evolves →
  Mitigation: keep comments at the module/design level, not tied to
  specific line numbers or function signatures, so they remain valid
  through minor refactors.

- [Risk] Explanations oversimplify Rust semantics →
  Mitigation: glosses are intentionally framed as "in this codebase, X is
  used to mean Y" rather than authoritative Rust definitions.

- [Risk] The two-part structure feels mechanical for simple modules →
  Mitigation: for very small modules (e.g., `ir/mod.rs`), the Design
  Decisions section may be brief (2–3 bullets) rather than forcing depth
  that isn't warranted.
