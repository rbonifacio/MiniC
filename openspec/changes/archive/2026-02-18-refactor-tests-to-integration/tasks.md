## 1. Library setup

- [x] 1.1 Create `src/lib.rs` with `pub mod ir;` and `pub mod parser;`
- [x] 1.2 Verify `cargo build` succeeds (binary will link against the new library)

## 2. Main refactor

- [x] 2.1 Remove `mod ir;` and `mod parser;` from `src/main.rs` (library owns these modules)
- [x] 2.2 Add `use minic::parser` (or equivalent) if `main` needs parser access; otherwise leave `main` as-is
- [x] 2.3 Verify `cargo run` succeeds

## 3. Integration tests

- [x] 3.1 Create `tests/parser.rs` (add `nom` as dev-dependency in Cargo.toml if tests need `all_consuming` or other nom combinators)
- [x] 3.2 Move literal tests from `literals.rs` to `tests/parser.rs` (use `minic::parser::literal`, `minic::ir::ast::Literal`)
- [x] 3.3 Move identifier tests from `identifiers.rs` to `tests/parser.rs`
- [x] 3.4 Move expression tests from `expressions.rs` to `tests/parser.rs`
- [x] 3.5 Move statement tests from `statements.rs` to `tests/parser.rs`
- [x] 3.6 Run `cargo test` and verify all 36 tests pass

## 4. Remove inline tests

- [x] 4.1 Remove `#[cfg(test)] mod tests { ... }` from `src/parser/literals.rs`
- [x] 4.2 Remove `#[cfg(test)] mod tests { ... }` from `src/parser/identifiers.rs`
- [x] 4.3 Remove `#[cfg(test)] mod tests { ... }` from `src/parser/expressions.rs`
- [x] 4.4 Remove `#[cfg(test)] mod tests { ... }` from `src/parser/statements.rs`
- [x] 4.5 Run `cargo test` again and verify all tests still pass
