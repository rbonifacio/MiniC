## Context

MiniC is used as a teaching tool. Its existing `doc/` tree was written
during implementation, mixing rationale useful to the author (technique
comparisons, academic references) with explanation useful to a student reader.
The recent source-level doc comment rewrite established a tone and structure
for student-facing content; the markdown docs now need to match it.

The primary constraint is audience: students who know programming fundamentals
but are new to compilers and to Rust. Every document must be independently
readable by someone who has just cloned the repo.

## Goals / Non-Goals

**Goals:**
- Every document has a concrete worked example before any mechanism is
  introduced.
- A student can read the documents in numbered order (01 → 08) and arrive at
  a complete mental model of the pipeline.
- Each document also stands alone — a student can jump directly to
  `05-type-checker.md` without having read 01–04.
- `README.md` is a true landing page: one sample program, three commands,
  eight links.
- No document references external concepts (Haskell, SmartPy) that require
  prior knowledge to decode.

**Non-Goals:**
- API reference documentation (that is the job of `cargo doc`).
- Documenting the `openspec/` workflow or change history.
- Rewriting tests or source code.

## Decisions

### D1 — Flat numbered `docs/` directory

**Choice:** All eight documents live directly in `docs/` with numeric
prefixes (`01-language.md` … `08-testing.md`). No subdirectories.

**Rationale:** The current `doc/architecture/` and `doc/design/` split is
meaningful to software architects but invisible to students. Numeric prefixes
encode the intended reading order without requiring a separate index document.
A student doing `ls docs/` immediately sees where to start.

**Alternative considered:** Subdirectories by topic (e.g., `docs/language/`,
`docs/internals/`). Rejected because it adds navigation friction and the
corpus is small enough (eight files) to not need grouping.

### D2 — Concrete example first, mechanism second

**Choice:** Every document opens with a small, complete, runnable example
relevant to that topic, then explains the mechanism the example illustrates.

**Rationale:** Students encountering a new concept need an anchor — something
concrete to refer back to as they read the abstract explanation. Starting with
the example sets that anchor before introducing terms like "parser combinator"
or "snapshot/restore".

**Alternative considered:** Concept-first, example-last (the current style).
Rejected because it requires understanding the concept before the example
makes sense, creating a circular dependency for the reader.

### D3 — Remove implementation archaeology from type-checker doc

**Choice:** Drop the Technique A/B/C/D comparison table and all Haskell/
SmartPy references from the type-checker document entirely.

**Rationale:** The comparison table documents decisions that were made during
implementation — it is a historical artefact, not an explanation. A student
reading it gains no understanding of *how the type checker works*; they only
learn that four approaches were considered. This information belongs in a
commit message or a design archive, not in learning material.

**Alternative considered:** Keep the table but add a disclaimer. Rejected
because the table would still pull focus away from the explanation.

### D4 — Document structure template

Every document follows the same internal structure so students develop a
reading habit:

1. **What is this?** — one paragraph, plain English, no jargon.
2. **Example** — a short, complete MiniC snippet or code trace.
3. **How it works** — the mechanism, referring back to the example.
4. **Key design decisions** — two or three decisions with one-sentence
   rationale each (not the full treatment — that lives in the source comments).
5. **What to read next** — a single line linking to the next document.

### D5 — `README.md` stays at project root

**Choice:** `README.md` is rewritten but stays at the root, not moved into
`docs/`.

**Rationale:** GitHub and most code hosts render the root `README.md`
automatically. Moving it would break the default project landing page.

## Risks / Trade-offs

- [Risk] Existing links inside `doc/` break when the directory is renamed →
  Mitigation: all cross-links are updated as part of the writing task; the
  old `doc/` directory is deleted only after all new files are in place.

- [Risk] Students skip `01-language.md` and start at `03-ast.md`, missing
  the language context → Mitigation: `README.md` and each document's
  "What to read next" section guide the recommended path explicitly.

- [Risk] Documents drift out of sync with source code as the project evolves
  → Mitigation: documents describe behaviour and concepts, not specific
  line numbers or function signatures, making them resilient to minor
  refactors.

## Migration Plan

1. Write all eight new documents in `docs/`.
2. Rewrite `README.md`.
3. Verify all cross-links between documents are correct.
4. Delete the `doc/` directory.
