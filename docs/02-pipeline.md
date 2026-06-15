# 2 — The MiniC Pipeline

This document explains how a MiniC source file goes from plain text to a
running program. There are five stages, each one transforming the program
into a more useful form.

---

## The Five Stages

```
Source text
    │
    ▼
┌─────────┐
│  Parser │  src/parser/
└─────────┘
    │  Unchecked AST
    ▼
┌──────────────┐
│ Type Checker │  src/semantic/
└──────────────┘
    │  Checked AST
    ▼
┌─────────────┐
│ Interpreter │  src/interpreter/
└─────────────┘
    │
    ▼
 Output
```

---

## Stage 1 — Parser

**Input:** A string of MiniC source code (e.g., the contents of a `.minic`
file).

**Output:** An *unchecked* Abstract Syntax Tree (AST) — a tree of Rust data
structures that represents the program's structure. At this stage, no type
information has been computed; each node in the tree carries an empty
placeholder instead of a type.

**What it does:** The parser reads the source text character by character
and recognises the MiniC grammar: function declarations, statements,
expressions, and literals. If the input does not match the grammar (e.g., a
missing parenthesis or an unknown keyword), the parser returns an error and
the pipeline stops.

The parser does *not* check whether variables have been declared, whether
function calls have the right number of arguments, or whether types are
compatible — those are semantic concerns handled in the next stage.

---

## Stage 2 — Type Checker

**Input:** The unchecked AST produced by the parser.

**Output:** Either a `TypeError` (on the first problem found) or a *checked*
AST — the same tree structure as before, but now every node carries its
inferred MiniC type (`int`, `float`, `bool`, etc.).

**What it does:** The type checker walks the unchecked AST and verifies that
the program is well-typed:

- Every variable is declared before it is used.
- Every expression has a consistent type (e.g., you cannot add an integer to
  a string).
- Every function call passes the right number of arguments of the right types.
- Every `return` statement returns a value compatible with the function's
  declared return type.
- A `void main()` function with no parameters exists.

Once the type checker succeeds, the output (the checked AST) is guaranteed to
be free of the errors listed above. The interpreter can trust this guarantee
and skip redundant checks at runtime.

---

## Stage 3 — Interpreter

**Input:** The checked AST.

**Output:** The side effects of running the program (printed output, values
read from stdin) and either a normal exit or a `RuntimeError`.

**What it does:** The interpreter walks the checked AST and *executes* it
directly. For every expression node it encounters, it computes a value. For
every statement node, it performs the corresponding action (declare a
variable, branch, loop, call a function). This approach is called a
*tree-walking* interpreter.

The interpreter can still encounter runtime errors even after type checking —
for example, dividing by zero or accessing an array element out of bounds.
These are not type errors (the types are correct), but they represent
conditions that cannot be known until the program actually runs.

---

## Why Three Separate Stages?

Each stage has a single, well-defined job. This makes the code smaller and
easier to understand — and easier to test — than a single monolithic
function that parses, checks, and executes all at once.

It also creates a useful compile-time guarantee: the interpreter only accepts
a *checked* AST. Rust's type system makes it impossible to accidentally pass
the parser's raw output straight to the interpreter, which would skip type
checking entirely.

---

**What to read next →** [03-ast.md](03-ast.md)
