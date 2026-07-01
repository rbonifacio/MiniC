## 1. Project Setup

- [x] 1.1 Create Rust project with `cargo init` (if not exists) and add `nom` dependency to Cargo.toml
- [x] 1.2 Create parser module structure: `src/parser/mod.rs`, `src/parser/literals.rs`, `src/parser/identifiers.rs`

## 2. Literal Types and Parsers

- [x] 2.1 Define `Literal` enum (or equivalent) in `parser/mod.rs` with variants for Int, Float, Str, Bool
- [x] 2.2 Implement integer literal parser in `literals.rs` (decimal digits, optional minus)
- [x] 2.3 Implement float literal parser in `literals.rs`
- [x] 2.4 Implement string literal parser in `literals.rs` with escapes `\"`, `\\`, `\n`, `\t`
- [x] 2.5 Implement boolean literal parser in `literals.rs` (`true`, `false`)
- [x] 2.6 Add combined `literal` parser that tries each literal type in order

## 3. Identifier Parser

- [x] 3.1 Implement identifier parser in `identifiers.rs` (letter/underscore start, alphanumeric + underscore)
- [x] 3.2 Ensure identifier parser rejects `true` and `false` (reserved words)

## 4. Tests

- [x] 4.1 Add unit tests for integer, float, string, and boolean literal parsers
- [x] 4.2 Add unit tests for identifier parser
- [x] 4.3 Run `cargo test` and verify all tests pass
