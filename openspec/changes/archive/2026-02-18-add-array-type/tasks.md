## 1. AST changes

- [x] 1.1 Add `Expr::ArrayLit(Vec<Expr>)` to `Expr` enum
- [x] 1.2 Add `Expr::Index { base: Box<Expr>, index: Box<Expr> }` to `Expr` enum
- [x] 1.3 Change `Stmt::Assign { target, value }` to `Stmt::Assign { target: Box<Expr>, value: Box<Expr> }` (target is lvalue)

## 2. Expression parser

- [x] 2.1 Add array literal to primary: `[ expr, expr, ... ]` (or empty `[]`)
- [x] 2.2 Add index postfix: after primary, allow zero or more `[ expr ]` suffixes, producing `Expr::Index`
- [x] 2.3 Order: array literal before identifier in primary; index as postfix on primary

## 3. Statement parser (assignment)

- [x] 3.1 Add `lvalue` parser: identifier followed by zero or more `[ expr ]` → produces `Expr`
- [x] 3.2 Change assignment to use lvalue: `lvalue = expr` → `Stmt::Assign { target: Box::new(lvalue), value }`
- [x] 3.3 Try indexed lvalue before simple identifier (or parse lvalue uniformly)

## 4. Update existing code

- [x] 4.1 Update all places that construct or match on `Stmt::Assign` (tests, etc.) for new target type

## 5. Tests and documentation

- [x] 5.1 Add tests: array literal, empty array, index read, index write, nested index, array in expression
- [x] 5.2 Update `doc/architecture/parser.md` for arrays
- [x] 5.3 Run `cargo test`
