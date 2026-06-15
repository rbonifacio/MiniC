# 6 — The Interpreter

The interpreter is the final stage of the pipeline. It takes the checked AST
produced by the type checker and executes it, producing output and side
effects.

---

## A Worked Example: Evaluating `2 + 3 * 4`

After parsing and type checking, `2 + 3 * 4` becomes this tree:

```
Add
├── Literal(2)
└── Mul
    ├── Literal(3)
    └── Literal(4)
```

The interpreter evaluates this tree by calling `eval_expr` recursively:

```
eval_expr(Add)
  ├── eval_expr(Literal(2))  →  Value::Int(2)
  └── eval_expr(Mul)
        ├── eval_expr(Literal(3))  →  Value::Int(3)
        └── eval_expr(Literal(4))  →  Value::Int(4)
        →  Int(3) * Int(4)  =  Value::Int(12)
  →  Int(2) + Int(12)  =  Value::Int(14)
```

Final result: `Value::Int(14)`.

This "walk the tree and compute a value at each node" approach is called a
**tree-walking interpreter**. It is the simplest kind of interpreter to build
and understand.

---

## Runtime Values: The `Value` Enum

At runtime, every MiniC value is represented by a variant of the `Value`
enum. In Rust, an `enum` can have *variants* that each carry different data —
think of it as a labelled box where the label tells you what type of value is
inside:

```rust
enum Value {
    Int(i64),           // an integer, e.g. Value::Int(42)
    Float(f64),         // a float,   e.g. Value::Float(3.14)
    Bool(bool),         // a boolean, e.g. Value::Bool(true)
    Str(String),        // a string,  e.g. Value::Str("hi".to_string())
    Array(Vec<Value>),  // a list,    e.g. Value::Array(vec![Int(1), Int(2)])
    Void,               // no value (returned by void functions)
    Fn(FnValue),        // a callable function
}
```

When the interpreter evaluates `42`, it returns `Value::Int(42)`. When it
evaluates `[1, 2, 3]`, it returns `Value::Array(vec![Int(1), Int(2), Int(3)])`.

### Functions as values: `FnValue`

Functions are stored in the environment alongside variables. There are two
kinds:

```rust
enum FnValue {
    UserDefined(CheckedFunDecl),  // a MiniC function — stores its AST
    Native(NativeFn),             // a Rust function — stores a function pointer
}
```

A **function pointer** (`NativeFn`) is a Rust variable that holds the address
of a specific function, rather than a data value. When the interpreter calls
`sqrt`, it looks up `"sqrt"` in the environment, gets back a
`Value::Fn(FnValue::Native(sqrt_fn))`, and calls `sqrt_fn` directly.

Both user-defined and native functions go through the same lookup path — the
interpreter does not need to know in advance which kind it is dealing with.

---

## Executing Statements

While expressions produce values, statements perform actions. The statement
executor `exec_stmt` handles each statement form:

| Statement | What happens |
|-----------|-------------|
| `int x = expr` | Evaluates `expr`, stores the result in the environment as `"x"` |
| `x = expr` | Evaluates `expr`, updates the existing binding for `"x"` |
| `{ stmt* }` | Executes each statement in order (with block scoping) |
| `if cond { s1 } else { s2 }` | Evaluates `cond`; executes `s1` or `s2` |
| `while cond { body }` | Repeatedly evaluates `cond` and executes `body` |
| `return expr` | Evaluates `expr` and signals an early return |
| `f(args)` | Evaluates arguments, calls `f`, discards the return value |

### How `return` propagates

Statements do not normally produce values, but `return` must pass its value
back through potentially many nested calls. The executor uses
`Result<Option<Value>, RuntimeError>` as its return type:

- `Ok(None)` — statement completed normally, keep going.
- `Ok(Some(v))` — a `return v` was hit; stop executing this block and pass
  `v` upward.
- `Err(e)` — a runtime error occurred.

Every block and loop checks for `Some(v)` and short-circuits immediately when
it sees one.

---

## Scoping

### Function calls: snapshot and restore

When a function is called, the interpreter needs to give it a clean scope:
parameter names must be bound, and the callee's local variables must not
interfere with the caller's variables.

The mechanism is simple: before the call, take a full clone of the
environment (`snapshot`). Bind the parameters. Run the body. After the call,
replace the environment with the saved clone (`restore`). Everything the
callee added disappears.

This also handles recursion correctly — each recursive call gets its own
parameter bindings because each call takes a fresh snapshot.

### Blocks: names and remove_new

Blocks (`{ … }`) require a different approach. Variables declared *inside* a
block must disappear when the block ends, but assignments to variables
declared *outside* the block must persist.

For example:

```c
int x = 0
{
  int y = 5;
  x = x + y    -- this assignment to outer x must survive the block
}
-- y is no longer visible here, but x is 5
```

The interpreter handles this by recording the set of names that exist *before*
the block (`names()`), running the block, then removing any name that was
*not* in that set (`remove_new`). The assignment to `x` updates the existing
binding and is not removed.

---

## Key Design Decision: Tree-Walking

The interpreter directly recurses over the AST without compiling to bytecode
or machine code first. This is the simplest approach: the code closely mirrors
the language semantics and is easy to follow. The trade-off is performance —
a bytecode VM would run programs faster — but for a teaching language with
small programs, simplicity matters more than speed.

---

**What to read next →** [07-stdlib.md](07-stdlib.md)
