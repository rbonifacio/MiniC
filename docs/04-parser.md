# 4 — The Parser

This document explains how the MiniC parser turns source text into an AST.
We start with a concrete example, then introduce the tools the parser uses.

---

## A Worked Example: Parsing `1 + 2`

Consider the expression `1 + 2`. The parser needs to produce the tree:

```
Add
├── Literal(1)
└── Literal(2)
```

Here is roughly what happens, step by step:

1. The parser calls the `expression` function with input `"1 + 2"`.
2. `expression` calls `additive`, which calls `multiplicative` to get the
   left operand.
3. `multiplicative` calls `unary`, which calls `primary`, which calls
   `atom`.
4. `atom` tries to match a literal — it sees `"1"` and returns
   `Literal(Int(1))`. The remaining input is `" + 2"`.
5. Back in `additive`: remaining input starts with `" + "`, so it matches
   the `+` operator and calls `multiplicative` again for the right side.
6. `multiplicative` → `unary` → `primary` → `atom` sees `"2"` and returns
   `Literal(Int(2))`. Remaining input is `""`.
7. `additive` builds `Expr::Add(Literal(1), Literal(2))` and returns it.

The same recursive logic handles arbitrarily complex expressions like
`a * (b + c) - sqrt(d)`.

---

## What is a Parser Combinator?

MiniC uses a library called **nom** to build the parser. `nom` provides
*combinators* — small functions that each recognise a tiny piece of syntax
and can be **composed** (combined) into parsers for larger constructs.

Every parser in `nom` follows the same contract:
- **Input:** a string slice `&str`
- **Output:** `IResult<&str, T>` — either `Ok((remaining, value))` on
  success, or `Err(...)` on failure

`remaining` is the part of the input that was not yet consumed. This is how
parsers chain: one parser hands its leftover input to the next.

---

## Key `nom` Combinators Used in MiniC

| Combinator | What it does | Example use |
|------------|-------------|-------------|
| `tag("x")` | Match the exact string `"x"` | `tag("if")` to match the keyword `if` |
| `char('x')` | Match the exact character `'x'` | `char('(')` for open parenthesis |
| `digit1` | Match one or more decimal digits | Integer literals |
| `alt((a, b, c))` | Try `a`; if it fails, try `b`; then `c` | Choice between statement forms |
| `tuple((a, b, c))` | Parse `a`, then `b`, then `c`; return all three | Parse keyword + name + body |
| `preceded(a, b)` | Parse `a` (discard), then parse `b` (keep) | Skip whitespace before a token |
| `delimited(a, b, c)` | Parse `a`, then `b`, then `c`; return only `b` | `(` expression `)` |
| `map(p, f)` | Parse with `p`, transform the result with function `f` | Build an AST node from raw text |
| `opt(p)` | Try `p`; return `Some(result)` or `None` | Optional `else` branch |
| `verify(p, pred)` | Parse with `p`; fail if `pred(result)` is false | Reject reserved words as identifiers |
| `many0(p)` | Apply `p` zero or more times; collect results | Parse all function declarations |

---

## Sub-module Decomposition

The parser is split into six files, each responsible for one grammatical
category. The dependencies flow from simple to complex:

```
program.rs        ← top-level entry point
  └── functions.rs    ← function declarations
        └── statements.rs   ← statements (if, while, block, …)
              └── expressions.rs  ← arithmetic, comparison, …
                    ├── literals.rs     ← numbers, strings, booleans
                    └── identifiers.rs  ← variable names
```

This mirrors how grammars are traditionally presented and makes it easy to
find the rule for any construct.

---

## Operator Precedence

The expression `1 + 2 * 3` must parse as `1 + (2 * 3)`, not `(1 + 2) * 3`.
This requires that `*` "binds tighter" than `+`.

MiniC encodes precedence through the **call chain**: each function calls the
next-higher-precedence function to get its operands. Higher in the chain =
tighter binding.

```
expression
    └── logical_or          (lowest precedence: or)
          └── logical_and   (and)
                └── logical_not   (!)
                      └── relational   (==  !=  <  <=  >  >=)
                            └── additive     (+  -)
                                  └── multiplicative   (*  /)
                                        └── unary      (unary -)
                                              └── primary   (atoms + indexing)
                                                    └── atom
```

When `additive` needs its right operand, it calls `multiplicative`. So `*`
always groups before `+` — naturally, without any precedence table.

**Why does this work?** Consider `1 + 2 * 3`:
1. `additive` reads `1` (via `multiplicative`).
2. It sees `+` and calls `multiplicative` again for the right side.
3. `multiplicative` reads `2 * 3` and returns `Mul(2, 3)` as a single unit.
4. `additive` builds `Add(1, Mul(2, 3))`. ✓

---

## Left-Associativity

For `1 - 2 - 3`, we want `(1 - 2) - 3`, not `1 - (2 - 3)`. This is called
*left-associativity*.

Each level that has left-associative operators uses an **accumulator loop**
instead of recursion:

```rust
fn additive(input) {
    let (rest, mut acc) = multiplicative(input)?;   // left-most operand
    loop {
        if let Ok((rest, right)) = parse("+" then multiplicative) {
            acc = Add(acc, right);   // fold left: (acc + right) becomes new acc
        } else if let Ok((rest, right)) = parse("-" then multiplicative) {
            acc = Sub(acc, right);
        } else {
            break;
        }
    }
    return acc;
}
```

For `1 - 2 - 3`:
- Start: `acc = 1`
- Loop 1: see `-2`, `acc = Sub(1, 2)`
- Loop 2: see `-3`, `acc = Sub(Sub(1, 2), 3)`
- Loop 3: no more `-` or `+`, stop.

Result: `(1 - 2) - 3` ✓

---

## Key Design Decisions

**No separate lexer.** Many compilers have two separate phases: a *lexer*
that turns characters into tokens (e.g., `"42"` → `Token::Int(42)`), then a
*parser* that processes tokens. MiniC skips the lexer and lets `nom` work
directly on characters. This is simpler for a small language and keeps the
code in one place.

**The parser produces an untyped AST.** Every node in the output carries
`ty: ()` — an empty placeholder. Type information is not the parser's
responsibility; it is computed in the separate type-checking stage. Keeping
parsing and type checking apart makes each phase shorter and independently
testable.

---

**What to read next →** [05-type-checker.md](05-type-checker.md)
