## 1. AST changes

- [ ] 1.1 Add `FunDecl { name, params, body }` to `ir/ast.rs`
- [ ] 1.2 Add `Expr::Call { name, args }` to `Expr` enum
- [ ] 1.3 Add `Stmt::Call { name, args }` to `Stmt` enum
- [ ] 1.4 Extend `Program` to `{ functions: Vec<FunDecl>, body: Vec<Stmt> }`

## 2. Function declaration parser

- [ ] 2.1 Create `parser/functions.rs` (or add to statements)
- [ ] 2.2 Implement `fun_decl` parser: `def` + identifier + `(` + param_list + `)` + statement
- [ ] 2.3 Implement param_list: comma-separated identifiers (possibly empty)
- [ ] 2.4 Export from parser mod

## 3. Function call in expressions

- [ ] 3.1 Add `call` parser: identifier + `(` + expr_list + `)`
- [ ] 3.2 Add call to `primary` in expressions.rs (before plain identifier)
- [ ] 3.3 Implement expr_list: comma-separated expressions

## 4. Function call as statement

- [ ] 4.1 Add `call_statement` parser
- [ ] 4.2 Add to `statement` alt: if | while | call | assignment
- [ ] 4.3 Order: try call (identifier + `(`) before assignment

## 5. Program parser (optional for this change)

- [ ] 5.1 If scope includes program parsing: add parser for `functions* body*`
- [ ] 5.2 Otherwise: document that Program structure is updated; program parsing can be a follow-up

## 6. Tests and documentation

- [ ] 6.1 Add tests for function declaration, call as expression, call as statement
- [ ] 6.2 Update `doc/architecture/parser.md` with functions and calls
- [ ] 6.3 Run `cargo test`
