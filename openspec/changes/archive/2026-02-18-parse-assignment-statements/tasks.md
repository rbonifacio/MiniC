## 1. Setup

- [x] 1.1 Create `src/parser/statements.rs` and add `pub mod statements` to `parser/mod.rs`
- [x] 1.2 Add `assignment` parser: `identifier` + `=` + `expression` producing `Stmt::Assign`

## 2. Export and Tests

- [x] 2.1 Export `assignment` from `parser/mod.rs`
- [x] 2.2 Add unit tests for assignment parser (simple, with expression, whitespace, invalid)
- [x] 2.3 Run `cargo test` and verify all tests pass
