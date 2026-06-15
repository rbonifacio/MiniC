## Why

The keyword `not` for logical negation is less familiar to developers coming from C-like languages. Changing to `!` aligns MiniC with common syntax (C, Java, Rust, JavaScript) and is more concise.

## What Changes

- Change the logical-not operator from keyword `not` to symbol `!` in the expression parser
- Update tests that use `not` to use `!`
- Update specs to reflect the new syntax

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `expressions`: Logical-not syntax changes from `not expr` to `!expr`

## Impact

- **Modified files**: `src/parser/expressions.rs`, `tests/parser.rs`, `openspec/specs/expressions/spec.md`
- **Breaking change**: Existing MiniC source using `not` will no longer parse; must migrate to `!`
