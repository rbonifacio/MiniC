# 3 — The Abstract Syntax Tree (AST)

This document explains what an AST is, how MiniC represents programs in
memory, and the clever trick that lets the same data structure serve both
the parser and the type checker.

---

## What is an AST?

When the parser reads source text like `x + 1 * 2`, it does not keep the
text as a string. Instead it builds a **tree** of data structures that
captures the program's structure:

```
    Add
   /   \
  x    Mul
       / \
      1   2
```

Each node in the tree is a Rust value. Leaf nodes are literals and variable
names; branch nodes are operations. This tree is called the *Abstract Syntax
Tree* (AST) because it captures the *abstract* structure of the code, not the
original characters.

The rest of the pipeline — type checker, interpreter — works entirely with
this tree.

---

## The Main Node Types

All AST types are defined in `src/ir/ast.rs`.

### `Expr` — an expression

An `Expr` is one of many variants. Think of it like a labelled box — each
label tells you what kind of expression it is, and the box holds whatever
data that expression needs:

```
Expr::Literal(42)            -- the integer 42
Expr::Ident("x")             -- the variable x
Expr::Add(left, right)       -- left + right
Expr::Mul(left, right)       -- left * right
Expr::Call { name, args }    -- a function call
Expr::Index { base, index }  -- base[index]
```

### `ExprD` — an expression *with a decoration*

Every expression node in the actual AST is wrapped in `ExprD`:

```rust
struct ExprD<Ty> {
    exp: Expr<Ty>,   // the expression itself
    ty:  Ty,         // the decoration slot
}
```

The `ty` field is the *decoration* — a slot that holds extra information
attached to the node. What goes in that slot depends on the pipeline stage
(more on this below).

### `Statement` and `StatementD`

The same pattern applies to statements: `Statement` holds the structure
(a `Decl`, `Assign`, `If`, `While`, etc.) and `StatementD` adds the
decoration slot.

### `FunDecl` — a function declaration

```rust
struct FunDecl<Ty> {
    name:        String,
    params:      Vec<(String, Type)>,
    return_type: Type,
    body:        Box<StatementD<Ty>>,
}
```

### `Program` — the whole program

```rust
struct Program<Ty> {
    functions: Vec<FunDecl<Ty>>,
}
```

A program is just a list of function declarations.

---

## The `Ty` Decoration: One Tree, Two Uses

The `<Ty>` in `ExprD<Ty>`, `Program<Ty>`, etc. is a **type parameter** — a
placeholder that you fill in with a concrete type when you use the struct.
Think of it as a sticky note holder attached to every node: the holder is
always there, but what you put on the sticky note changes between stages.

### After parsing: `Ty = ()` (empty)

The parser does not know types yet. So it fills every decoration slot with
`()` — the empty tuple, which carries no information and takes no memory.
A parsed expression has type `ExprD<()>`.

### After type checking: `Ty = Type` (full)

The type checker fills every decoration slot with the inferred MiniC type.
A type-checked expression has type `ExprD<Type>`, where `Type` is one of
`Type::Int`, `Type::Float`, `Type::Bool`, etc.

The same tree structure, the same node types — just a different value in the
`ty` field.

---

## Type Aliases

To avoid writing `Program<()>` and `Program<Type>` everywhere, the code
defines readable aliases:

| Alias | Full type | Produced by |
|-------|-----------|-------------|
| `UncheckedProgram` | `Program<()>` | Parser |
| `CheckedProgram` | `Program<Type>` | Type checker |
| `UncheckedExpr` | `ExprD<()>` | Parser |
| `CheckedExpr` | `ExprD<Type>` | Type checker |
| `UncheckedStmt` | `StatementD<()>` | Parser |
| `CheckedStmt` | `StatementD<Type>` | Type checker |

The function signatures in the pipeline use these aliases:

```rust
// Type checker: takes unchecked, returns checked or an error
fn type_check(program: &UncheckedProgram) -> Result<CheckedProgram, TypeError>

// Interpreter: only accepts checked — the compiler won't let you pass unchecked
fn interpret(program: &CheckedProgram) -> Result<(), RuntimeError>
```

---

## Why This Design?

### One definition, not two

The alternative would be to define two completely separate sets of types:
`Expr`, `TypedExpr`, `Statement`, `TypedStatement`, etc. That works, but it
doubles the code and means that any change to the tree structure has to be
made in two places.

By using a single definition with a type parameter, there is one `Expr` to
maintain. The compiler automatically creates the two versions (`Expr<()>` and
`Expr<Type>`) from the single definition.

### The compiler enforces the pipeline order

Because `interpret` takes a `CheckedProgram` and `type_check` returns a
`CheckedProgram`, it is literally impossible in Rust to call `interpret` on
the raw output of the parser — the types do not match. The compiler catches
this mistake at build time, before any tests even run.

---

**What to read next →** [04-parser.md](04-parser.md)
