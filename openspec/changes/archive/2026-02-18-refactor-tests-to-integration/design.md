## Context

MiniC is currently a binary-only crate. Parser tests live in `#[cfg(test)] mod tests` blocks inside `literals.rs`, `identifiers.rs`, `expressions.rs`, and `statements.rs`. Rust recommends integration tests in `tests/` for testing the public API. To use `tests/`, the crate must expose a library.

## Goals / Non-Goals

**Goals:**

- Add `src/lib.rs` exposing `ir` and `parser` as the library API
- Move all parser tests to `tests/parser.rs` as integration tests
- Keep `main.rs` as a thin binary that uses the library
- Preserve test coverage; no behavior change

**Non-Goals:**

- Splitting tests across multiple files (e.g., `tests/literals.rs`, `tests/expressions.rs`) — single `parser.rs` is sufficient for now
- Adding new tests or changing test logic
- Adding `dev-dependencies` or test utilities

## Decisions

### 1. Library structure: `lib.rs` re-exports modules

**Choice:** Create `src/lib.rs` with `pub mod ir; pub mod parser;`. The library root mirrors the current structure; no module reorganization.

**Rationale:** Minimal change. `main.rs` and tests both `use minic::parser::*` etc. Crate name from Cargo.toml is `MiniC`; the library is used as `minic` (Rust lowercases crate names in `use`).

### 2. `main.rs` uses the library

**Choice:** Replace `mod ir; mod parser;` in `main.rs` with `use minic::parser` (or nothing if `main` does not yet use the parser). Keep `main` as a thin entry point.

**Rationale:** The binary crate links against the library. `main.rs` currently only prints; no parser usage yet. Once `main` needs the parser, it will `use minic::parser::*` or the specific items.

### 3. Single `tests/parser.rs` for all parser tests

**Choice:** One `tests/parser.rs` file containing all parser tests, grouped by module (literals, identifiers, expressions, statements) via `mod` blocks or `#[test]` functions.

**Rationale:** Keeps the refactor simple. All 36 tests fit in one file. Can split later if it grows.

### 4. Public API: expose what tests need

**Choice:** Tests use `minic::parser::{literal, identifier, expression, assignment, statement}` and `minic::ir::ast::{Expr, Literal, Stmt}`. These are already `pub` in the parser modules. No new `pub` items required.

**Rationale:** Tests only need the public API. If any test currently uses private items, we either make them `pub(crate)` or adjust the test to use the public API.
