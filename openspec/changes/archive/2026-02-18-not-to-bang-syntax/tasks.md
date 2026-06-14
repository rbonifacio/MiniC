## 1. Parser change

- [x] 1.1 In `expressions.rs`, change `tag("not")` to `char('!')` in `logical_not`
- [x] 1.2 Update the comment for `logical_not` to say `!` instead of `not`

## 2. Tests and specs

- [x] 2.1 Update `test_boolean_expr` in `tests/parser.rs`: change `not x` to `!x`
- [x] 2.2 Run `cargo test` and verify all tests pass
- [x] 2.3 Sync delta spec to main `openspec/specs/expressions/spec.md`
