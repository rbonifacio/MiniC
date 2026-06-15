## Why

The MiniC codebase lacks meaningful module-level documentation, making it hard
for students — the primary audience — to understand the architecture and design
intent without reading every line of source code. Adding structured doc comments
now, while the recent interpreter and environment refactors are fresh, will turn
the codebase into a learning resource rather than just an implementation.

## What Changes

- Add `//!` (module-level) doc comments to every module entry point across all
  six subsystems: `ir`, `parser`, `semantic`, `environment`, `interpreter`, and
  `stdlib`.
- Each comment covers two things: (1) a high-level overview of the module's
  purpose and the main types/functions it exposes, and (2) a design decisions
  section explaining the key architectural choices made in that module.
- Language and tone are aimed at students who know programming fundamentals but
  are not Rust experts. Rust-specific constructs (enums with data, generic type
  parameters, function pointers via `fn` types, trait implementations) are
  explained in plain terms where they first appear.
- No changes to any source code logic, types, or tests.

## Capabilities

### New Capabilities

- `module-documentation`: Doc comments for all modules, written for a student
  audience, covering purpose, exposed API, and design rationale.

### Modified Capabilities

<!-- None — this change adds documentation only; no spec-level behavior changes. -->

## Impact

- **Files touched:** `src/ir/mod.rs`, `src/parser/mod.rs`,
  `src/semantic/mod.rs`, `src/environment/mod.rs`, `src/interpreter/mod.rs`,
  `src/stdlib/mod.rs`, plus sub-module files where design decisions are best
  explained inline (`interpreter/eval_expr.rs`, `interpreter/exec_stmt.rs`,
  `environment/env.rs`, `stdlib/math.rs`, `stdlib/io.rs`).
- **No API or behavioral changes.** Existing tests remain unaffected.
- **Audience constraint:** Every design decision explanation must be
  self-contained — students should not need to know Rust idioms in advance to
  understand the rationale.
