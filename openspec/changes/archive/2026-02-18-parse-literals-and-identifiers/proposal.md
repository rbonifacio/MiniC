## Why

The MiniC parser needs to recognize the atomic building blocks of expressions before it can parse arithmetic, relational, or boolean expressions. Literals (integers, floats, strings, booleans) and identifiers (variable names) are the leaves of the expression tree. Implementing these parsers first establishes the foundation for all subsequent parsing work in the MiniC language.

## What Changes

- Add nom-based parsers for integer literals (e.g., `42`, `-17`)
- Add nom-based parsers for float literals (e.g., `3.14`, `-0.5`)
- Add nom-based parsers for string literals (e.g., `"hello"`, `"a\"b"`)
- Add nom-based parsers for boolean literals (`true`, `false`)
- Add nom-based parser for identifiers (variable names, e.g., `x`, `count`, `_temp`)
- Define AST types or value types for parsed literals and identifiers in Rust

## Capabilities

### New Capabilities

- `literals`: Parsing of integer, float, string, and boolean literals in MiniC source text
- `identifiers`: Parsing of variable identifiers (names) in MiniC source text

### Modified Capabilities

- (none)

## Impact

- **New Rust modules**: Parser module(s) for literals and identifiers using the nom crate
- **Dependencies**: Requires `nom` (and possibly `nom_locate` if source positions are needed)
- **Project structure**: Establishes the initial parser layout for the MiniC Rust project
