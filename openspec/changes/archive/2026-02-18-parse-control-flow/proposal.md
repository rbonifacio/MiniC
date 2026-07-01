## Why

MiniC programs need conditional and loop constructs. Parsing `if expr then stmt [else stmt]` and `while expr do stmt` produces `Stmt::If` and `Stmt::While` nodes, enabling control flow in the AST.

## What Changes

- Add `if` and `while` parsers in `parser/statements.rs`
- Parse `if expr then stmt [else stmt]` producing `Stmt::If`
- Parse `while expr do stmt` producing `Stmt::While`
- Add unified `statement` parser (assignment | if | while) for mutual recursion

## Capabilities

### New Capabilities

- `control-flow`: Parsing of if-then-else and while statements in MiniC, producing `Stmt::If` and `Stmt::While` nodes.

### Modified Capabilities

- `assignment-statements`: Now composed under `statement`; `statement` is the entry point for parsing any statement.

## Impact

- **Modified module**: `src/parser/statements.rs` (add if, while, statement)
- **Dependencies**: Uses `parser::expression`, `ir::ast`; mutually recursive with assignment
- **Integration**: Completes statement parsing; parse-assignment-statements established the pattern
