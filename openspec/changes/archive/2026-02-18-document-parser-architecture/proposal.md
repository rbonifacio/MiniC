## Why

The parser is a core component of MiniC. Documenting its design helps maintainers understand how parsing works, how combinators compose, and how operator precedence is achieved. A step-by-step presentation makes the concepts accessible to readers new to parser combinators or nom.

## What Changes

- Create `doc/architecture/` folder
- Add `doc/architecture/parser.md` documenting the parser component
- Use a step-by-step style: parsing concepts → combinators → nom combinators → operator precedence

## Capabilities

### New Capabilities

- `parser-docs`: Architecture documentation for the parser in `doc/architecture/parser.md`

### Modified Capabilities

- (none)

## Impact

- **New files**: `doc/architecture/parser.md`
- **No code changes**: Documentation only
