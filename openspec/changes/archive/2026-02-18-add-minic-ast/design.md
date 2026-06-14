## Context

MiniC is a minimal C-like language with integers, floats, strings, booleans; expressions (arithmetic, relational, boolean); and statements (assignment, if-then-else, while). The parser is being built in Rust using nom. The AST is the output of parsing and the input to future phases (semantic analysis, code generation). It must be defined before the parser can produce AST nodes; it is currently implemented in `src/ir/ast.rs`.

## Goals / Non-Goals

**Goals:**

- Define AST types in `ir/ast.rs` that satisfy the spec
- Use `Box` for recursive types to avoid infinite size
- Derive `Debug` and `PartialEq` for all AST types

**Non-Goals:**

- Parsing (handled by parser modules)
- Semantic analysis or code generation
- Source location or span information (can be added later)

## Decisions

### 1. Use `Box` for recursive types

**Choice:** `Expr` and `Stmt` use `Box<Expr>` and `Box<Stmt>` for recursive nesting.

**Rationale:** Rust requires a known size for recursive types. `Box` allocates on the heap and provides a fixed-size pointer.

**Alternative considered:** `Rc` or `Arc` for shared ownership. Rejected; AST is typically built once and consumed, not shared.

### 2. Owned `String` for identifiers

**Choice:** `Ident(String)` and `Assign { target: String, ... }` use owned `String`.

**Rationale:** AST outlives the parser input; identifiers must be owned. `String` is simple and avoids lifetime complexity.

**Alternative considered:** `&str` with lifetimes. Rejected; would require AST to borrow from source, complicating storage and APIs.

### 3. Single `Expr` enum (no separate `ArithmeticExpr`, etc.)

**Choice:** One `Expr` enum with variants for all expression kinds.

**Rationale:** Simpler type system; expression kinds are mutually exclusive. Pattern matching is straightforward.

**Alternative considered:** Separate enums per expression kind. Rejected; would require wrapper types and more boilerplate.

### 4. `Option<Box<Stmt>>` for else-branch

**Choice:** Else-branch is `Option<Box<Stmt>>` for optional else.

**Rationale:** Directly models "if optional else" in the grammar.

### 5. `Program` as `Vec<Stmt>`

**Choice:** `Program { statements: Vec<Stmt> }` for the root.

**Rationale:** A program is a sequence of statements. A struct allows future fields (e.g., metadata) without breaking changes.

## Risks / Trade-offs

| Risk | Mitigation |
|------|-------------|
| No source locations | Add `Span` or `Loc` to nodes in a later change if error reporting needs it |
| No `Clone` for large ASTs | Add `#[derive(Clone)]` if needed; `Box` and `Vec` are `Clone` when their contents are |
| String interning | Not needed; identifiers are small; can add later if needed for performance |
