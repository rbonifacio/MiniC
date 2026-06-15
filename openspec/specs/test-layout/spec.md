# Test Layout

## Purpose

Integration tests in `tests/` following Rust conventions; library structure with `lib.rs` exposing parser and IR.

## Requirements

### Requirement: Library structure

The crate SHALL expose a library via `src/lib.rs`. The library SHALL expose the `ir` and `parser` modules as the public API.

#### Scenario: Library compiles and is reachable

- **WHEN** the project is built with `cargo build`
- **THEN** the library SHALL compile without errors and SHALL be linkable by the binary target

#### Scenario: Parser and IR are public

- **WHEN** an external crate or the binary uses `minic::parser` or `minic::ir`
- **THEN** the parser functions (`literal`, `identifier`, `expression`, `assignment`, `statement`) and AST types (`Expr`, `Literal`, `Stmt`) SHALL be accessible

---

### Requirement: Integration tests in tests/

The parser tests SHALL live in `tests/parser.rs` as integration tests. Integration tests SHALL use only the public API of the library.

#### Scenario: Tests run from tests/ directory

- **WHEN** `cargo test` is run
- **THEN** the tests in `tests/parser.rs` SHALL execute and SHALL pass

#### Scenario: No inline unit tests in parser modules

- **WHEN** the source files `literals.rs`, `identifiers.rs`, `expressions.rs`, and `statements.rs` are inspected
- **THEN** they SHALL NOT contain `#[cfg(test)] mod tests` blocks

#### Scenario: Test coverage preserved

- **WHEN** `cargo test` is run
- **THEN** the number of passing tests SHALL be at least the number that passed before the refactor (36 tests)

---

### Requirement: Binary uses the library

The binary target (`src/main.rs`) SHALL use the library rather than defining modules directly.

#### Scenario: Main does not define ir or parser

- **WHEN** `main.rs` is inspected
- **THEN** it SHALL NOT contain `mod ir;` or `mod parser;` (those SHALL be in `lib.rs`)

#### Scenario: Binary compiles and runs

- **WHEN** `cargo run` is executed
- **THEN** the binary SHALL build and run without errors
