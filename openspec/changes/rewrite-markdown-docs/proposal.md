## Why

The existing markdown documentation was written from the perspective of an
implementer and contains academic references (Haskell GADTs, SmartPy,
technique comparisons) that are inaccessible to students encountering a
compiler pipeline for the first time. Now that the source-level doc comments
have been updated for a student audience, the markdown docs should follow the
same principle: lead with examples, explain concepts before mechanisms, and
provide a clear guided reading path.

## What Changes

- **BREAKING** Rename `doc/` to `docs/` and flatten the subdirectory
  structure (remove `architecture/` and `design/` subdirs).
- Rewrite `README.md` as a true entry point: one-paragraph description, a
  complete sample MiniC program, three build/run/test commands, and a table
  linking to the eight docs with one-line descriptions.
- **Remove** `doc/summary.md` — its content is absorbed into `docs/02-pipeline.md`.
- Add `docs/01-language.md` (new) — the MiniC language from a user
  perspective: types, statement forms, operator precedence, a complete
  factorial example. No implementation detail.
- Rewrite `docs/02-pipeline.md` (from `doc/summary.md`) — five-stage visual
  diagram and one paragraph per stage. No Rust code.
- Rewrite `docs/03-ast.md` (from `doc/architecture/ast.md`) — keep the
  parameterised AST concept, replace Haskell/SmartPy references with
  self-contained Rust-based explanations.
- Rewrite `docs/04-parser.md` (from `doc/architecture/parser.md`) —
  restructure to lead with a worked example before introducing `nom`.
- Rewrite `docs/05-type-checker.md` (from `doc/design/type-checker.md`) —
  drop the Technique A/B/C/D comparison table entirely; replace with a
  walkthrough of three concrete type errors and how they are caught.
- Rewrite `docs/06-interpreter.md` (from `doc/architecture/interpreter.md`)
  — add a concrete eval trace of `2 + 3 * 4` as the opening example.
- Rewrite `docs/07-stdlib.md` (from `doc/architecture/stdlib.md`) — add a
  step-by-step guide for adding a new native function.
- Rewrite `docs/08-testing.md` (from `doc/architecture/tests.md`) — add a
  worked example of writing a new test end to end; update to reflect that
  stdlib tests now live in `tests/stdlib.rs`.

## Capabilities

### New Capabilities

- `student-docs`: A complete, student-oriented documentation suite covering
  the MiniC language, the five-stage pipeline, and each implementation
  module, with a guided reading order and concrete worked examples throughout.

### Modified Capabilities

<!-- None — existing openspec/specs/ describe parser and language behaviour,
     not documentation requirements. No spec-level behaviour changes. -->

## Impact

- `doc/` directory: deleted and replaced by `docs/`.
- `README.md`: rewritten in place.
- No source code, tests, or openspec specs are affected.
- All internal cross-links between docs must be updated to the new paths.
