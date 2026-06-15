## Context

The MiniC parser is implemented in Rust using the nom library. It parses literals, identifiers, expressions, and statements into an AST. The expression parser uses a precedence-climbing approach with recursive descent. Documentation should explain the design to readers who may be new to parser combinators.

## Goals / Non-Goals

**Goals:**

- Create `doc/architecture/parser.md` with step-by-step exposition
- Introduce parsing concepts, combinators, and nom combinators progressively
- Explain operator precedence implementation in detail

**Non-Goals:**

- API reference (that's rustdoc)
- Tutorial on nom itself (link to nom docs instead)

## Decisions

### 1. Location: `doc/architecture/parser.md`

**Choice:** Place under `doc/architecture/` as specified. The `doc/` folder is a common Rust convention for project documentation.

**Rationale:** Keeps architecture docs separate from API docs; `architecture` signals design-level content.

### 2. Step-by-step structure

**Choice:** Order sections as: (1) What is parsing, (2) Combinators, (3) Nom combinators used, (4) Parser structure, (5) Operator precedence (detailed).

**Rationale:** Builds from fundamentals to implementation; precedence gets its own section for emphasis.

### 3. Code snippets from actual implementation

**Choice:** Include real code snippets from `src/parser/` with brief annotations.

**Rationale:** Readers can cross-reference with the source; examples are accurate.
