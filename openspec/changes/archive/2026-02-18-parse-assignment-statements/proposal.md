## Why

MiniC programs need to parse assignment statements (`id = expr`) so that variable assignments can be represented in the AST. This is the first statement type in the parser and enables programs that modify state.

## What Changes

- Add assignment statement parser in `parser/statements.rs` (or `parser/assignments.rs`)
- Parse `identifier = expression` producing `ir::ast::Stmt::Assign`
- Compose existing identifier and expression parsers

## Capabilities

### New Capabilities

- `assignment-statements`: Parsing of assignment statements (`id = expr`) in MiniC, producing `ir::ast::Stmt::Assign` nodes.

### Modified Capabilities

- (none)

## Impact

- **New module**: `src/parser/statements.rs` (or dedicated `assignments.rs`)
- **Dependencies**: Uses `parser::identifiers`, `parser::expression`, and `ir::ast`
- **Integration**: Establishes statement parsing; parse-control-flow will add if/while and compose with this
