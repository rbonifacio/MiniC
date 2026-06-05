## Why

MiniC currently exposes only `while` for iteration. A traditional three-part
`for (init; cond; update) body` is far more expressive for counted loops and is
the feature required by Projeto 2 in `docs/09-projects.md`. Milestone 1 of that
project covers the concrete syntax, AST, and parser.

## What Changes

- Extend `src/ir/ast.rs` with a new `Statement::For { init, cond, update, body }`
  variant where `init`, `cond`, and `update` are optional.
- Add a `for_statement` parser in `src/parser/statements.rs` recognising
  `for '(' [init] ';' [expression] ';' [update] ')' block`, where `init` is
  an optional declaration/assignment (without trailing `;`) and `update` is
  an optional assignment (also without trailing `;`). The body must be a
  block, matching the existing `if`/`while` convention.
- Register `for_statement` in the `statement` dispatcher before `decl_statement`
  and `assignment` so the `for` keyword is matched unambiguously.
- Add `"for"` to the `RESERVED` list in `src/parser/identifiers.rs` so it cannot
  be used as a variable name.
- Add integration tests in `tests/parser.rs` covering simple, nested,
  assign-initialised, whitespace, and invalid-form cases.

## Capabilities

### New Capabilities

- `for-statement`: Parsing of `for ([init]; [condition]; [update]) body`
  statements in MiniC, including `for (;;)`.

### Modified Capabilities

- `ast`: Adds a `Statement::For` variant alongside the existing statement nodes.

## Impact

- **Modified modules**:
  - `src/ir/ast.rs` — new `Statement::For` variant.
  - `src/parser/statements.rs` — new `for_statement` parser plus the
    no-semicolon init/update helpers; dispatcher updated.
  - `src/parser/identifiers.rs` — reserve `for` keyword.
  - `tests/parser.rs` — integration tests for the new grammar.
- **Dependencies**: Uses `parser::expression`, `parser::identifiers`,
  `parser::functions::type_name`, and `ir::ast`. Mutually recursive with
  `statement` for the body.
- **Integration**: Purely additive. All existing programs and tests continue to
  parse identically. Type checker and interpreter work for the `For` variant is
  out of scope and will be addressed in Milestone 2.
