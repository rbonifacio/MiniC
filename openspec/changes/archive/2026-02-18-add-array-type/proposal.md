## Why

MiniC has no aggregate types. Arrays enable storing and indexing sequences of values (e.g., `arr[i]`, `[1, 2, 3]`), which is essential for many programs.

## What Changes

- **AST**: Add `Expr::ArrayLit(Vec<Expr>)`, `Expr::Index { base, index }`; extend `Stmt::Assign` to accept an lvalue (Ident or Index) as target
- **Parser**: Add array literal `[ expr, ... ]`, index expression `expr[expr]`, and lvalue parsing for assignment
- **Grammar**: Array literals in primary; indexing as postfix; assignment target = lvalue
- **Documentation**: Update `doc/architecture/parser.md` for arrays

## Capabilities

### New Capabilities

- `array-literals`: `[1, 2, 3]` or `[a, b, c]` in expressions
- `array-indexing`: `arr[i]` for reading; `arr[i] = x` for writing
- `multi-dimensional`: `arr[i][j]` via recursive Index

### Modified Capabilities

- `ast`: Add `Expr::ArrayLit`, `Expr::Index`; change `Stmt::Assign` target from `String` to `Expr` (lvalue)
- `expressions`: Add array literal, index postfix
- `statements`: Assignment accepts lvalue (identifier or indexed)
- `parser-docs`: Document array parsing

## Impact

- **Modified**: `ir/ast.rs`, `parser/expressions.rs`, `parser/statements.rs`, `doc/architecture/parser.md`, `tests/parser.rs`
- **Breaking**: `Stmt::Assign { target }` changes from `String` to `Box<Expr>` (lvalue)
