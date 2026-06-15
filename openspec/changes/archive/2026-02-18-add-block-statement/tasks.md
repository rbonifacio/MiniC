## 1. AST changes

- [x] 1.1 Add `Stmt::Block { seq: Vec<Stmt> }` to `Stmt` enum in `ir/ast.rs`

## 2. Block parser

- [x] 2.1 Add `block_statement` parser in `parser/statements.rs`: `{` + separated statements + `}`
- [x] 2.2 Use `separated_list0` (or similar) for statements separated by `;`
- [x] 2.3 Add block to `statement` alt: if | while | call | block | assignment
- [x] 2.4 Order: block before assignment (block starts with `{`, unambiguous)

## 3. Function body

- [x] 3.1 No change needed: `fun_decl` already uses `statement` for body; block is now one alternative

## 4. Tests and documentation

- [x] 4.1 Add tests for: empty block, single-statement block, multi-statement block, block in function body, block in if/while
- [x] 4.2 Update `doc/architecture/parser.md` with block statement
- [x] 4.3 Run `cargo test`
