## Context

MiniC is a minimal C-like language supporting integers, floats, strings, and booleans. The parser is being built in Rust using the nom framework. This change implements the foundational parsers for literals and identifiers—the atomic elements that expressions are built from. There is no existing parser code; this is greenfield.

## Goals / Non-Goals

**Goals:**

- Implement nom parsers for integer, float, string, and boolean literals
- Implement nom parser for identifiers (variable names)
- Define Rust types for parsed values (AST or value representation)
- Use nom combinators for composable, zero-copy parsing where practical

**Non-Goals:**

- Parsing expressions, statements, or full programs
- Lexer/tokenizer as a separate phase (nom will parse directly from `&str`)
- Error recovery or detailed error messages (basic parse success/failure is sufficient for now)

## Decisions

### 1. Parse directly from `&str` with nom (no separate lexer)

**Choice:** Use nom's `&str` input type and character-level combinators to parse literals and identifiers directly.

**Rationale:** For a small language, a combined lexer-parser approach is simpler. nom handles whitespace via combinators (e.g., `multispace0`). A separate token stream would add complexity without clear benefit at this stage.

**Alternative considered:** Separate lexer producing tokens, then parser consuming tokens. Rejected for added complexity.

### 2. Return typed values, not raw strings

**Choice:** Parsers return Rust types (e.g., `i64`, `f64`, `String`, `bool`, `&str` for identifiers) rather than raw string slices.

**Rationale:** Downstream expression parsing needs typed values. Converting at parse time avoids repeated conversion logic.

**Alternative considered:** Return `&str` and convert later. Rejected because conversion belongs at parse boundaries.

### 3. Identifier rules: letter or underscore start, alphanumeric + underscore

**Choice:** Identifiers MUST start with a letter or underscore; subsequent characters MAY be letters, digits, or underscores. Reserved words (`true`, `false`) are excluded.

**Rationale:** Matches common C-style identifier rules. Boolean literals are parsed before identifiers to avoid `true`/`false` being misparsed as identifiers.

### 4. String escape sequences

**Choice:** Support `\"`, `\\`, `\n`, `\t` in string literals. Other escapes (e.g., `\xNN`) are out of scope for this change.

**Rationale:** Covers the most common cases. Extensible later if needed.

### 5. Module layout

**Choice:** Organize as `src/parser/literals.rs` and `src/parser/identifiers.rs`, with a `src/parser/mod.rs` re-exporting public parsers.

**Rationale:** Keeps literals and identifiers separate for clarity; shared types (e.g., `Literal`) can live in `parser/mod.rs` or a `parser/ast.rs`.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Float parsing ambiguity (e.g., `1.` vs `1`) | Use nom's `double` or define explicit grammar; document accepted formats |
| Identifier vs keyword collision | Parse `true`/`false` before attempting identifier in expression context; document ordering |
| No source locations yet | Omit `nom_locate` for now; add in a later change if error reporting needs it |
