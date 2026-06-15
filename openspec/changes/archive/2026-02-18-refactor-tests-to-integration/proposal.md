## Why

Parser tests are currently embedded in each source file (`#[cfg(test)] mod tests`). Moving them to the standard Rust `tests/` directory improves separation of concerns, keeps implementation files focused, and follows the recommended layout for integration tests. It also prepares the crate for future consumers (e.g., a REPL or CLI) that will use the library.

## What Changes

- Add `src/lib.rs` to expose `ir` and `parser` as the library API
- Refactor `main.rs` to use the library (e.g., `use minic::parser::*`)
- Create `tests/parser.rs` with integration tests for literals, identifiers, expressions, and statements
- Remove `#[cfg(test)] mod tests` blocks from `literals.rs`, `identifiers.rs`, `expressions.rs`, and `statements.rs`

## Capabilities

### New Capabilities

- `test-layout`: Integration tests in `tests/` following Rust conventions; library structure with `lib.rs` exposing parser and IR.

### Modified Capabilities

- (none)

## Impact

- **New files**: `src/lib.rs`, `tests/parser.rs`
- **Modified files**: `src/main.rs`, `src/parser/literals.rs`, `src/parser/identifiers.rs`, `src/parser/expressions.rs`, `src/parser/statements.rs`
- **No behavior change**: Tests remain equivalent; only location and style change
- **Crate structure**: Binary crate becomes library + binary (standard Rust layout)
