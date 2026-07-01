## Why

The expression parser should handle realistic complex expressions combining relational operators, parentheses, multiplication, and addition. Adding a test for `a >= (pi * r * r) + epsilon` validates that the parser correctly handles such expressions (area comparison with tolerance).

## What Changes

- Add an integration test in `tests/parser.rs` for the expression `a >= (pi * r * r) + epsilon`
- Verify the parser produces the expected AST structure (Ge with Ident left, Add of parenthesized Mul on right)

## Capabilities

### New Capabilities

- (none — extends existing expression test coverage)

### Modified Capabilities

- `expressions`: Additional test scenario for complex expression parsing.

## Impact

- **Modified file**: `tests/parser.rs`
- **No parser changes**: Only adds a test; parser already supports this expression form
