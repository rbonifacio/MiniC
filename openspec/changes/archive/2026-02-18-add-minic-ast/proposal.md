## Why

The MiniC parser produces structured output that downstream phases (semantic analysis, code generation) consume. An explicit AST type establishes the contract between parsing and later stages, enables testing and debugging, and keeps the IR layer separate from parser implementation details.

## What Changes

- Add `src/ir/` module with `ast.rs` containing the MiniC AST definitions
- Define `Literal` enum (Int, Float, Str, Bool)
- Define `Expr` enum (literals, identifiers, arithmetic, relational, boolean operations)
- Define `Stmt` enum (Assign, If, While)
- Define `Program` struct (sequence of statements)
- All AST types derive `Debug` and `PartialEq`

## Capabilities

### New Capabilities

- `ast`: Abstract syntax tree for MiniC, including literal/identifier nodes, expression nodes (arithmetic, relational, boolean), statement nodes (assignment, if-then-else, while), and program root. Implemented in `ir/ast.rs`.

### Modified Capabilities

- (none)

## Impact

- **New module**: `src/ir/mod.rs`, `src/ir/ast.rs`
- **Dependencies**: Parser will depend on `ir::ast` when producing AST nodes (future changes)
- **No breaking changes**: This is additive
