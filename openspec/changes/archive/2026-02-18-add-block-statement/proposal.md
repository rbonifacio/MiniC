## Why

MiniC functions currently accept only a single statement as body. To implement functions with multiple statements (e.g., `def foo(x, y) { x = x + 1; y = y + 1 }`), we need a way to group statements. A block is the standard construct for this.

## What Changes

- **AST**: Add `Stmt::Block { seq: Vec<Stmt> }` to group a sequence of statements
- **Parser**: Add block parser for `{ stmt* }` syntax (statements separated by `;` or newline)
- **Grammar**: Allow function body to be a block; optionally allow blocks in `if`/`while` bodies
- **Documentation**: Update `doc/architecture/parser.md` for blocks

## Capabilities

### New Capabilities

- `blocks`: Block statement `{ stmt ; stmt ; ... }` for grouping multiple statements
- `multi-statement-functions`: Function bodies can be blocks, enabling multi-statement functions

### Modified Capabilities

- `ast`: Add `Stmt::Block`
- `statements`: Add block statement; function body accepts block
- `parser-docs`: Document block parsing

## Impact

- **Modified**: `ir/ast.rs`, `parser/statements.rs`, `parser/functions.rs`, `doc/architecture/parser.md`
- **Breaking**: None (additive change)
