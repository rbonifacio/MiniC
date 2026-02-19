# Parser Architecture

This document describes the MiniC parser: how it works, the concepts it builds on, and how operator precedence is implemented. We proceed step by step from parsing fundamentals to the concrete design.

---

## 1. What is Parsing?

**Parsing** is the process of turning a stream of characters (source code) into a structured representation. For MiniC, that representation is an *abstract syntax tree* (AST): a tree of nodes representing literals, expressions, statements, and so on.

Two common approaches:

- **Lexer + parser**: A lexer first turns characters into tokens (e.g., `"42"` → `INT(42)`); the parser then consumes tokens. Many production compilers use this.
- **Parser combinators**: The parser works directly on characters, combining small parsers into larger ones. No separate lexer. MiniC uses this approach with the [nom](https://github.com/rust-bakery/nom) library.

Parser combinators are well-suited for small languages: they keep the implementation concise, and the structure of the grammar is visible in the code.

---

## 2. Combinators: Building Blocks

A **combinator** is a function that takes one or more parsers and returns a new parser. The idea: start with tiny parsers (e.g., "parse a single digit") and combine them into parsers for larger constructs (e.g., "parse an integer").

Examples of combinators:

- **Sequence**: Parse A, then B. If both succeed, combine their results.
- **Choice (alternation)**: Try parser A; if it fails, try parser B.
- **Repetition**: Parse A zero or more times.
- **Map**: Parse A, then transform the result with a function.

Combinators are *composable*: the output of one can be the input of another. This lets us build a full parser from small, reusable pieces.

---

## 3. Nom Combinators Used in MiniC

[nom](https://github.com/rust-bakery/nom) is a parser combinator library for Rust. MiniC uses these combinators:

| Combinator | Purpose | Example |
|------------|---------|---------|
| `tag("x")` | Parse the exact string `"x"` | `tag("if")` for the `if` keyword |
| `char('x')` | Parse the exact character `'x'` | `char('!')` for logical not |
| `digit1` | Parse one or more decimal digits | Integer literals |
| `take_while1(p)` | Take one or more chars satisfying predicate `p` | Identifier start |
| `take_while(p)` | Take zero or more chars satisfying `p` | Identifier rest |
| `alt((a, b, c))` | Try `a`; if it fails, try `b`, then `c` | Choice among alternatives |
| `tuple((a, b, c))` | Parse `a` then `b` then `c`, return `(Ra, Rb, Rc)` | Sequence |
| `preceded(a, b)` | Parse `a` (discard) then `b` | Whitespace before a token |
| `delimited(a, b, c)` | Parse `a`, then `b`, then `c`; return result of `b` | Parentheses: `delimited('(', expr, ')')` |
| `map(p, f)` | Parse `p`, apply `f` to the result | Convert parsed value to AST node |
| `opt(p)` | Parse `p` zero or one time | Optional `else` branch |
| `value(v, p)` | Parse `p`, return `v` on success | `value(true, tag("true"))` |
| `verify(p, pred)` | Parse `p`, succeed only if `pred(result)` holds | Reject `"true"` as identifier |
| `recognize(p)` | Parse `p`, return the matched slice (no transformation) | Get identifier string |
| `pair(a, b)` | Parse `a` then `b`, return `(Ra, Rb)` | Two-part sequence |
| `escaped_transform` | Parse with escape sequences, transform escapes | String literals |

The parser returns `IResult<&str, T>`: either `Ok((remaining_input, value))` or `Err(...)`. The `remaining_input` is the slice of the input that was not consumed.

---

## 4. Parser Structure

The MiniC parser is organized into modules:

```
src/parser/
├── mod.rs          # Re-exports
├── literals.rs     # Integers, floats, strings, booleans
├── identifiers.rs  # Variable names
├── expressions.rs  # Arithmetic, relational, boolean expressions
├── statements.rs   # Assignment, if, while, call, block
├── functions.rs    # Function declarations
├── program.rs      # Top-level program: functions* body*
```

### 4.1 Literals

Literals are the simplest atoms. Each type has its own parser; `literal` tries them in order:

```rust
pub fn literal(input: &str) -> IResult<&str, Literal> {
    alt((
        map(boolean_literal, Literal::Bool),
        map(integer_literal, Literal::Int),
        map(float_literal, Literal::Float),
        map(string_literal, Literal::Str),
    ))(input)
}
```

Order matters: `"true"` must match boolean before integer; `"42"` must match integer before float.

### 4.2 Identifiers

Identifiers use `recognize` and `verify`:

```rust
let id_parser = recognize(pair(
    take_while1(|c| c.is_alphabetic() || c == '_'),
    take_while(|c| c.is_alphanumeric() || c == '_'),
));
verify(id_parser, |s| s != "true" && s != "false")(input)
```

`recognize` returns the matched slice; `verify` rejects reserved words.

### 4.3 Statements

Statements use `alt` for choice and `tuple` for sequences:

```rust
pub fn statement(input: &str) -> IResult<&str, Stmt> {
    preceded(multispace0, alt((
        if_statement,
        while_statement,
        call_statement,
        block_statement,
        assignment,
    )))(input)
}
```

`if_statement` and `while_statement` call `statement` recursively for their bodies, enabling nested control flow. `call_statement` parses a function call at statement level (e.g., `foo(1, 2)`). `block_statement` parses `{ stmt ; stmt ; ... }` and produces `Stmt::Block { seq }`; it enables multi-statement function bodies and compound if/while bodies.

### 4.4 Functions

Function declarations use the syntax `def name(params) body`:

```rust
pub fn fun_decl(input: &str) -> IResult<&str, FunDecl> {
    preceded(multispace0, tuple((
        tag("def"),
        multispace1,
        identifier,
        delimited(char('('), separated_list0(...), char(')')),
        preceded(multispace0, statement),
    )))(input)
}
```

The body is a single statement. Parameters are comma-separated identifiers; the list may be empty.

### 4.5 Function Calls

Function calls appear in two places:

1. **As expressions** (in `primary`): `identifier(args)` is tried before a plain identifier. So `foo(1, 2)` parses as `Expr::Call { name, args }`, not `Expr::Ident`.
2. **As statements**: `call_statement` uses `parse_call` to parse `name(args)` and produces `Stmt::Call`.

`parse_call` returns `(String, Vec<Expr>)` and is shared by both expression and statement parsers.

### 4.6 Program

The top-level parser produces a `Program { functions, body }`:

```rust
pub fn program(input: &str) -> IResult<&str, Program> {
    map(
        tuple((many0(fun_decl), many0(statement))),
        |(functions, body)| Program { functions, body },
    )(input)
}
```

It parses zero or more function declarations, then zero or more statements (the main body). Use `all_consuming(program)` to require that the entire input is consumed.

---

## 5. Operator Precedence

Expressions like `1 + 2 * 3` must parse as `1 + (2 * 3)`, not `(1 + 2) * 3`. That requires **operator precedence**: `*` binds tighter than `+`.

### 5.1 Precedence Levels

The expression parser is structured as a chain of functions, each handling one precedence level. Lower precedence calls higher precedence for its operands:

| Level | Function | Operators | Binds |
|-------|----------|-----------|-------|
| Highest | `primary` | literals, `identifier(args)` (call), identifiers, `(expr)` | — |
| | `unary` | `-` | — |
| | `multiplicative` | `*`, `/` | left |
| | `additive` | `+`, `-` | left |
| | `relational` | `==`, `!=`, `<`, `<=`, `>`, `>=` | left |
| | `logical_not` | `!` | — |
| | `logical_and` | `and` | left |
| Lowest | `logical_or` | `or` | left |

The key rule: **a level parses its operands using the next higher level**. So `additive` parses `multiplicative` as the operands of `+` and `-`. That means `1 + 2 * 3` is parsed as: `additive` sees `1` (from `multiplicative`), then `+`, then `2 * 3` (a full `multiplicative`). Result: `1 + (2 * 3)`.

### 5.2 The Call Chain

```
expression  →  logical_or  →  logical_and  →  logical_not  →  relational
                                                                    ↓
                                                              additive
                                                                    ↓
                                                              multiplicative
                                                                    ↓
                                                                unary
                                                                    ↓
                                                                primary
```

When `logical_or` needs an operand, it calls `logical_and`. When `additive` needs an operand, it calls `multiplicative`. This nesting enforces precedence.

### 5.3 Left-Associativity

For `1 - 2 - 3`, we want `(1 - 2) - 3`, not `1 - (2 - 3)`. That's **left-associativity**.

The additive parser uses a loop that accumulates the left side:

```rust
fn additive(input: &str) -> IResult<&str, Expr> {
    let (mut rest, mut acc) = multiplicative(input)?;   // acc = leftmost operand
    loop {
        let add = tuple((multispace0, tag("+"), multispace0, multiplicative))(rest);
        if let Ok((r, (_, _, _, e))) = add {
            acc = Expr::Add(Box::new(acc), Box::new(e));  // acc = (acc + e)
            rest = r;
            continue;
        }
        let sub = tuple((multispace0, tag("-"), multispace0, multiplicative))(rest);
        if let Ok((r, (_, _, _, e))) = sub {
            acc = Expr::Sub(Box::new(acc), Box::new(e));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}
```

For `1 - 2 - 3`:

1. `acc = 1` (from first `multiplicative`)
2. Parse `-` and `2`; `acc = Sub(1, 2)`
3. Parse `-` and `3`; `acc = Sub(Sub(1, 2), 3)`
4. No more `+` or `-`; return `acc`

The same pattern applies to `multiplicative`, `relational`, `logical_and`, and `logical_or`.

### 5.4 Unary `!` and Recursion

Logical not uses `alt` and recursion:

```rust
fn logical_not(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            pair(
                preceded(multispace0, char('!')),
                preceded(multispace0, logical_not),   // recursive call
            ),
            |(_, e)| Expr::Not(Box::new(e)),
        ),
        relational,
    ))(input)
}
```

`!expr` is parsed by matching `!` and then recursively parsing `logical_not` for the operand. That allows `!!x`. The base case is `relational` (no `!`).

### 5.5 Parentheses

Parentheses are handled in `primary`:

```rust
delimited(
    preceded(multispace0, char('(')),
    preceded(multispace0, expression),   // full expression inside
    preceded(multispace0, char(')')),
)
```

`(1 + 2) * 3` works because `primary` parses `(1 + 2)` as a full expression, then `multiplicative` sees that as the left operand of `*` and `3` as the right.

---

## 6. Tests

The parser is covered by two test suites:

### 6.1 Unit tests (`tests/parser.rs`)

Unit tests parse individual constructs (literals, identifiers, expressions, statements, functions, blocks) from inline strings. They verify each parser in isolation and cover edge cases (e.g., invalid identifiers, unbalanced parentheses).

### 6.2 Program integration tests (`tests/program.rs`)

Integration tests parse complete MiniC programs from fixture files in `tests/fixtures/`. They read `.minic` files, parse them with `all_consuming(program)`, and assert on the resulting `Program` structure.

| Fixture | Content | Purpose |
|---------|---------|---------|
| `empty.minic` | (empty) | Empty program |
| `statements_only.minic` | `x = 1` and `y = 2` | Body only, no functions |
| `function_single.minic` | `def foo() x = 1` | Single function, no body |
| `function_with_block.minic` | `def add(x, y) { x = x + y; x = x }` | Function with block body |
| `full_program.minic` | Two functions and body with calls | Complete program |
| `invalid_syntax.minic` | Incomplete `def foo(` | Invalid input; parse must fail |

Tests use `env!("CARGO_MANIFEST_DIR")` so fixture paths work regardless of the working directory. Input is trimmed before parsing to handle trailing newlines.

---

## 7. Summary

- **Parsing** turns source text into an AST.
- **Combinators** combine small parsers into larger ones.
- **Nom** provides combinators like `alt`, `tuple`, `map`, `preceded`, `delimited`.
- **Precedence** is encoded by the call chain: each level calls the next higher level for operands.
- **Left-associativity** is achieved by a loop that accumulates the left operand and repeatedly extends it with `op(acc, right)`.
- **Tests** are split into unit tests (inline strings) and integration tests (fixture files).

For more on nom, see the [nom documentation](https://docs.rs/nom/).
