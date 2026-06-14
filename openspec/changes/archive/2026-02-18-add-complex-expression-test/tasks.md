## 1. Add test

- [x] 1.1 Add `test_complex_expression` in `tests/parser.rs` for `a >= (pi * r * r) + epsilon`
- [x] 1.2 Assert AST structure: Ge(Ident("a"), Add(Mul(...), Ident("epsilon")))
- [x] 1.3 Run `cargo test` and verify the new test passes
