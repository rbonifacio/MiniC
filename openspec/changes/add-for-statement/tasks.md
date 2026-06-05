## 1. AST

- [ ] 1.1 Add `Statement::For { init, cond, update, body }` variant to
      `Statement<Ty>` in `src/ir/ast.rs`, with `init`/`update`/`body` typed as
      `Box<StatementD<Ty>>` and `cond` as `Box<ExprD<Ty>>`.

## 2. Parser

- [ ] 2.1 Add `decl_no_semi` and `assign_no_semi` helpers in
      `src/parser/statements.rs` that mirror `decl_statement` and `assignment`
      but stop before the `;`.
- [ ] 2.2 Add `for_init_clause` (alt of `decl_no_semi` / `assign_no_semi`) and
      `for_update_clause` (only `assign_no_semi`).
- [ ] 2.3 Add `for_statement` parser:
      `'for' '(' init ';' expression ';' update ')' block`.
- [ ] 2.4 Register `for_statement` in the `statement` dispatcher, before
      `decl_statement` and `assignment`.

## 3. Reserved keyword

- [ ] 3.1 Add `"for"` to the `RESERVED` list in
      `src/parser/identifiers.rs`.

## 4. Tests

- [ ] 4.1 Add a `// --- For ---` section in `tests/parser.rs` with:
      `test_simple_for`, `test_for_with_assign_init`, `test_nested_for`,
      `test_for_inside_function`, `test_for_whitespace`,
      `test_invalid_for_missing_parens`, `test_invalid_for_missing_semis`,
      `test_invalid_for_bare_body`, `test_invalid_for_update_not_assign`,
      `test_for_reject_as_identifier`.
- [ ] 4.2 Run `cargo build` and `cargo test`; verify all existing tests still
      pass.

## 5. Pull-request

- [ ] 5.1 Open the Milestone 1 pull-request referencing this change proposal.
