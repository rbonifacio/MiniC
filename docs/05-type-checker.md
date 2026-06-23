# 5 — The Type Checker

The type checker is the second stage of the pipeline. It reads the
unchecked AST produced by the parser and either reports the first type error
it finds, or returns a checked AST where every node is annotated with its
MiniC type.

---

## Three Errors the Type Checker Catches

Before diving into how it works, here are three concrete examples of what the
type checker prevents.

### Error 1: Using a variable before declaring it

```c
void main() {
  x = 10
}
```

Error: `undeclared variable: x`

`x` was never declared with a type (`int x = …`), so the type checker does
not know what type it has, and rejects the program.

### Error 2: Type mismatch in an expression

```c
void main() {
  int x = 1;
  bool flag = x + true
}
```

Error: `arithmetic operands must be Int or Float`

`true` is a `bool`, not a number. Adding a number and a boolean has no
defined meaning in MiniC, so the type checker rejects it.

### Error 3: Wrong number of arguments to a function

```c
int add(int x, int y)
  return x + y

void main() {
  int result = add(1, 2, 3)
}
```

Error: `function 'add' expects 2 arguments, got 3`

`add` takes exactly two parameters. Calling it with three is a mistake the
type checker catches before the program ever runs.

---

## How the Type Checker Works

The entry point is:

```rust
fn type_check(program: &UncheckedProgram) -> Result<CheckedProgram, TypeError>
```

It walks every function in the program and, for each one, walks every
statement and expression in its body, computing and checking types as it goes.

### Tracking names with `Environment<Type>`

To know the type of a variable or function at any point in the program, the
type checker keeps a **symbol table** — a map from names to their types. This
is the same `Environment<V>` struct used by the interpreter, but here
instantiated with `V = Type` instead of `V = Value`:

- When the type checker sees `int x = 5`, it adds `"x" → Type::Int` to the
  environment.
- When it later sees `x + 1`, it looks up `"x"` in the environment, gets
  `Type::Int`, and checks that `Int + Int` is valid (it is — result is `Int`).

Functions are stored in the same environment as variables. A function
`int add(int x, int y)` is stored as `"add" → Type::Fun([Int, Int], Int)`.

### Registering all functions first

Before checking any function body, the type checker registers the type
signatures of *all* functions (including built-in stdlib functions). This
means functions can call each other — even in mutual recursion — without
needing forward declarations. The registration happens in two steps:

1. All stdlib functions are registered (e.g., `"print" → Fun([Any], Unit)`).
2. All user-defined functions from the program are registered.
3. A snapshot (`fn_snapshot`) is taken of the environment at this point —
   only function names, no variable names.

Then, for each function body:
- The environment is **restored** to `fn_snapshot` (clearing any variables
  left over from the previous function).
- The function's parameters are added.
- The body is type-checked.

This ensures variable bindings from one function never accidentally "leak"
into another.

### Block scoping

When the type checker enters a block `{ … }`, variables declared inside the
block must not be visible outside it. This is handled with a snapshot:

```
before block: take env snapshot
  check each statement (new variables are added to env)
after block: restore to snapshot (new variables disappear)
```

Importantly, this restores only *new* bindings. If a statement inside the
block assigns to a variable declared *outside* the block, that outer variable
keeps its type unchanged — the snapshot approach preserves the outer type.

### Int/Float coercion

MiniC allows mixing `int` and `float` in arithmetic:

| Left | Right | Result |
|------|-------|--------|
| `int` | `int` | `int` |
| `int` | `float` | `float` |
| `float` | `int` | `float` |
| `float` | `float` | `float` |

The type checker applies these rules automatically. `1 + 3.14` is legal and
has type `float`.

### `Type::Any` for `print`

The built-in `print` function must accept any value type. Rather than
adding special logic everywhere, `print` is registered with a parameter type
of `Type::Any`. The type checker's compatibility check treats `Any` as
matching every type, so `print(42)`, `print(true)`, and `print([1,2,3])` all
pass type checking.

`Type::Any` is never inferred for a variable or expression — it only appears
in the registry as a parameter type for built-in functions.

### `len` and `contains` as expression forms

`len` and `contains` are no longer validated through stdlib function
signatures. They are checked as dedicated expression nodes:

- `len(expr)`
: `expr` must be `str` or `array`, result type is `int`.
- `contains(container, item)`
: if `container` is `str`, `item` must be `str`; if `container` is
  `array(T)`, `item` must be compatible with `T`; result type is `bool`.

This moves type errors for those constructs to their specific expression
rules, instead of generic call-argument validation.

---

## Key Design Decision: Fail on the First Error

The type checker stops and reports the first error it encounters. It does not
collect multiple errors and report them all at once. This simplifies the
implementation and is appropriate for a teaching language where programs are
short and students typically fix one error at a time.

---

**What to read next →** [06-interpreter.md](06-interpreter.md)
