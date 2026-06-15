## Context

MiniC has if-then-else and while statements. The AST defines `Stmt::If` and `Stmt::While`. The assignment parser exists in `statements.rs`. This change adds if and while parsing; all statement types must be mutually recursive (body of if/while can be any statement).

## Goals / Non-Goals

**Goals:**

- Parse `if expr then stmt [else stmt]` producing `Stmt::If`
- Parse `while expr do stmt` producing `Stmt::While`
- Support nested control flow (if in if, while in while, assignment in both)

**Non-Goals:**

- Statement sequences or block parsing (`{ stmt; stmt }`)
- Program parsing (sequence of statements)
- Semantic checks (e.g., condition must be boolean)

## Decisions

### 1. Grammar: Pascal-like keywords

**Choice:** Use `if expr then stmt [else stmt]` and `while expr do stmt`. Keywords: `if`, `then`, `else`, `while`, `do`.

**Rationale:** Matches common MiniC variants; avoids C-style braces and parentheses; distinct keywords reduce ambiguity.

### 2. Mutual recursion via `statement` parser

**Choice:** Define `statement` that parses assignment | if | while. Use `statement` for the body of if/while. Define `if_statement` and `while_statement` that call `statement` for their bodies.

**Rationale:** Body of if/while can be any statement (including nested if/while). Nom handles recursion; we avoid left-recursion by having assignment/if/while as alternatives.

### 3. Parsing order: if, while, assignment

**Choice:** Try `if` first, then `while`, then `assignment` in the `statement` alt.

**Rationale:** Keywords `if` and `while` are unambiguous; assignment starts with identifier. Trying keyword-based first avoids identifier consuming `if` or `while` as variable names.
