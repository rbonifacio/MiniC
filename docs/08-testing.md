# 8 — Testing

This document explains how the MiniC test suite is organised and how to add
new tests.

---

## Overview

MiniC has two complementary testing layers:

| Layer | Tool | What it tests |
|-------|------|---------------|
| Library tests | `cargo test` | Parser, type checker, interpreter, stdlib — via the Rust API |
| CLI tests | `shelltest` | Binary behaviour — flags, output, exit codes |

```
tests/
├── parser.rs        # Parser tests (literals, expressions, statements, …)
├── program.rs       # Full-program parsing from fixture files
├── type_checker.rs  # Type-checking tests
├── interpreter.rs   # End-to-end execution tests
├── stdlib.rs        # Standard library function tests
├── fixtures/        # MiniC source files shared by both layers
│   ├── empty.minic
│   ├── function_single.minic
│   ├── function_with_block.minic
│   ├── full_program.minic
│   ├── invalid_syntax.minic
│   ├── cli_type_mismatch.minic
│   └── …
└── cli/             # shelltestrunner test files
    ├── check.test   # --check flag scenarios
    ├── run.test     # --run flag scenarios
    └── errors.test  # bad arguments and missing files
```

Run both layers:

```bash
cargo build          # required before shelltest (tests the debug binary)
cargo test           # Rust library tests
shelltest tests/cli/ # CLI tests
```

---

## CLI Tests with shelltestrunner

[shelltestrunner](https://github.com/simonmichael/shelltestrunner) (`shelltest`)
is a small Haskell tool that reads plain-text `.test` files and checks that a
command produces the expected stdout, stderr, and exit code. It is ideal for
testing CLI programs because the tests are language-agnostic, easy to read, and
require no test harness code.

### Test file format

Each test is a short block:

```
# optional comment — used as the test name in output
$ command to run
expected stdout (verbatim, or omit if none)
>2 expected stderr line (or >2 /regex/)
>= expected exit code
```

Only the fields you specify are checked. Omitting `>=` skips exit-code
validation; omitting `>2` skips stderr validation.

### The three test files

**`tests/cli/check.test`** — `--check` flag scenarios:

| Scenario | Expected |
|----------|----------|
| Valid program | `'<file>' is well-typed.` on stdout, exit 0 |
| Type error | Type error message on stderr, exit 1 |
| Malformed program | Type error on stderr, exit 1 |

**`tests/cli/run.test`** — `--run` flag scenarios:

| Scenario | Expected |
|----------|----------|
| `interpreter_hello.minic` | `42 / true / hello` on stdout, exit 0 |
| `interpreter_factorial.minic` | `120 / 1 / 1` on stdout, exit 0 |
| `interpreter_array.minic` | `10 / 99 / 30` on stdout, exit 0 |
| Program with type error | Type error on stderr, exit 1 |

**`tests/cli/errors.test`** — bad invocations:

| Scenario | Expected |
|----------|----------|
| No arguments | Usage message on stderr, exit 1 |
| Unknown flag | Usage message on stderr, exit 1 |
| Flag without file | Usage message on stderr, exit 1 |
| Non-existent file | File error on stderr, exit 1 |

### Running shelltest

```bash
shelltest tests/cli/         # run all CLI tests
shelltest tests/cli/ -c      # with colour output
shelltest tests/cli/ -j4     # in parallel (4 threads)
shelltest tests/cli/run.test # one file only
```

shelltest is included in the Nix dev shell (`flake.nix`). After `direnv allow`
or `nix develop`, `shelltest` is available on your PATH.

### Adding a new CLI test

1. Add (or reuse) a `.minic` fixture in `tests/fixtures/`.
2. Open the relevant `.test` file in `tests/cli/` (or create a new one).
3. Append a test block:

```
# describe what this tests
$ ./target/debug/mini_c --run tests/fixtures/my_program.minic
expected output line 1
expected output line 2
>=0
```

4. Run `shelltest tests/cli/` to verify it passes.

---

## The Five Test Files

### `tests/parser.rs` — Parser Tests

Tests individual parser functions (`literal`, `expression`, `statement`,
`fun_decl`, etc.) using **inline strings**. Each test focuses on one small
construct and verifies either the AST node produced or that invalid input
is rejected.

Parser functions return `IResult<&str, T>`: `Ok((remaining_input, value))`
on success or `Err(…)` on failure. Tests use `assert_eq!` for the success
case and `assert!(…is_err())` for the failure case.

Expression and statement parsers return `ExprD<()>` or `StatementD<()>`.
To compare just the expression shape (ignoring the `ty: ()` field), map
over the result:

```rust
#[test]
fn test_primary_literal() {
    assert_eq!(
        expression("42").map(|(r, e)| (r, e.exp)),
        Ok(("", Expr::Literal(Literal::Int(42))))
    );
}
```

When you need to verify that *all* input is consumed (not just the prefix),
wrap the parser in `all_consuming(…)`:

```rust
assert!(all_consuming(expression)("1 + 2)").is_err());
```

---

### `tests/program.rs` — Full-Program Parsing Tests

Tests parsing of **complete MiniC programs** from fixture files in
`tests/fixtures/`. Use this when the program is long enough that an inline
string would be hard to read, or when you want to reuse a program across
multiple tests.

The helper `parse_program_file("name.minic")` reads the fixture, parses it
with `all_consuming(program)`, and returns a `Result<UncheckedProgram, …>`.

```rust
#[test]
fn test_parse_function_single() {
    let prog = parse_program_file("function_single.minic")
        .expect("should parse");
    assert_eq!(prog.functions.len(), 1);
    assert_eq!(prog.functions[0].name, "foo");
}

#[test]
fn test_parse_invalid_syntax_fails() {
    assert!(parse_program_file("invalid_syntax.minic").is_err());
}
```

---

### `tests/type_checker.rs` — Type-Checking Tests

Tests the type checker with short programs as inline strings. Each test
parses a program and then type-checks it, asserting either success or a
specific type error.

A helper `parse_and_type_check(src)` combines both steps:

```rust
#[test]
fn test_type_check_undeclared_var() {
    let result = parse_and_type_check("void main() x = y");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("undeclared"));
}

#[test]
fn test_type_check_int_float_coercion() {
    assert!(parse_and_type_check(
        "void main() { int x = 1; float y = x + 3.14 }"
    ).is_ok());
}
```

---

### `tests/interpreter.rs` — End-to-End Tests

Tests the complete pipeline: parse → type-check → interpret. These are the
highest-level tests, and they verify that programs produce correct output or
correct runtime errors.

A helper `run(src)` runs the full pipeline and returns `Ok(())` or
`Err(RuntimeError)`. Output side effects (e.g., `print`) are not captured
by the test — these tests mainly check return status and runtime errors:

```rust
#[test]
fn test_factorial() {
    assert!(run(
        "int factorial(int n)
           if n <= 1 then return 1 else return n * factorial(n - 1)
         void main() {
           int r = factorial(10);
           print(r)
         }"
    ).is_ok());
}

#[test]
fn test_out_of_bounds() {
    let result = run(
        "void main() { int[] a = [1, 2]; int x = a[5] }"
    );
    assert!(result.is_err());
}
```

---

### `tests/stdlib.rs` — Standard Library Tests

Tests the built-in functions directly, without going through the full
pipeline. These are fast, focused unit-style tests for `print_fn`, `pow_fn`,
`sqrt_fn`, and `NativeRegistry`.

```rust
#[test]
fn test_pow_int_args() {
    let result = pow_fn(vec![Value::Int(2), Value::Int(10)]);
    assert_eq!(result, Ok(Value::Float(1024.0)));
}

#[test]
fn test_default_registry_contains_all_stdlib() {
    let r = NativeRegistry::default();
    assert!(r.lookup("print").is_some());
    assert!(r.lookup("sqrt").is_some());
}
```

---

## Inline Strings vs Fixture Files

| Use | When |
|-----|------|
| Inline string | Short, focused test; program fits in one or two lines |
| Fixture file | Multi-line program; program shared across several tests |

Fixture files go in `tests/fixtures/` with a `.minic` extension.

---

## Worked Example: Adding a New Interpreter Test

Suppose you have added a `min(int, int) → int` function to the interpreter
and want to test it end to end.

**Step 1** — Open `tests/interpreter.rs`.

**Step 2** — Add a test that defines `min` in a MiniC program and checks
the result:

```rust
#[test]
fn test_stdlib_min() {
    assert!(run(
        "int min(int a, int b)
           if a <= b then return a else return b
         void main() {
           int r = min(3, 7);
           print(r)
         }"
    ).is_ok());
}
```

**Step 3** — Run `cargo test test_stdlib_min` to verify your test passes.

If you also want to add a unit test for `min` as a native stdlib function,
open `tests/stdlib.rs` and follow the same pattern as `test_pow_int_args`.

---

## Naming Conventions

- Test functions: `test_<construct>_<scenario>`
  e.g., `test_integer_positive`, `test_if_with_else`, `test_out_of_bounds`
- Fixture files: descriptive, lowercase, with `.minic` extension
  e.g., `function_with_block.minic`, `factorial.minic`

---

**What to read next →** [README.md](../README.md)
