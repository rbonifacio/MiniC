## 1. If statement

- [x] 1.1 Add `if_statement` parser: `if` + expr + `then` + statement + optional (`else` + statement)
- [x] 1.2 Produce `Stmt::If { cond, then_branch, else_branch }`

## 2. While statement

- [x] 2.1 Add `while_statement` parser: `while` + expr + `do` + statement
- [x] 2.2 Produce `Stmt::While { cond, body }`

## 3. Statement dispatcher

- [x] 3.1 Add `statement` parser: if_statement | while_statement | assignment
- [x] 3.2 Export `statement` from `parser/mod.rs`

## 4. Tests

- [x] 4.1 Add unit tests for if (with/without else, nested, invalid)
- [x] 4.2 Add unit tests for while (simple, nested, invalid)
- [x] 4.3 Run `cargo test` and verify all tests pass
