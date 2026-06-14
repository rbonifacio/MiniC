## Why

MiniC needs functions to support reusable logic. Function declarations define named procedures with parameters; function calls invoke them. Calls appear both as expressions (returning a value) and as standalone statements (for side effects).

## What Changes

- **AST**: Add `FunDecl` (name, params, body), `Expr::Call` (name, args), `Stmt::Call` (or use `Expr::Call` in expression-statement)
- **Parser**: Add function declaration parser; add call to `primary` (expression); add call statement
- **Program**: Extend to hold function declarations (e.g., `functions: Vec<FunDecl>` before body)
- **Documentation**: Update `doc/architecture/parser.md` for functions and calls

## Capabilities

### New Capabilities

- `functions`: Function declarations (`def name(params) body`) and function calls (`name(args)`) as expressions and statements.

### Modified Capabilities

- `ast`: Add `FunDecl`, `Expr::Call`; extend `Program`
- `expressions`: Add call to primary expressions
- `statements`: Add call statement
- `parser-docs`: Document function/call parsing

## Impact

- **Modified**: `ir/ast.rs`, `parser/expressions.rs`, `parser/statements.rs`, `parser/mod.rs`, `doc/architecture/parser.md`
- **New**: `parser/functions.rs` (or integrate into statements)
- **Breaking**: `Program` structure changes
