# MiniC

MiniC is a small C-like programming language implemented in Rust. It is
designed as a teaching tool: the entire pipeline — parser, type checker, and
interpreter — is intentionally small so you can read and understand every part
of it. A complete MiniC program looks like this:

```c
int factorial(int n) {
  if n <= 1 { return 1; }
  return n * factorial(n - 1);
}

void main() {
  int result = factorial(10);
  print(result);
}
```

---

## Quick Start

```bash
cargo build          # compile the project
cargo test           # run the Rust library tests
shelltest tests/cli/ # run the CLI tests (requires cargo build first)
```

---

## Nix Development Environment

This project ships a [Nix flake](https://nixos.wiki/wiki/Flakes) that pins
every tool to a reproducible version. If you have Nix installed you never need
to manage Rust, Clippy, or shelltestrunner yourself — the flake does it for
you.

### What is Nix?

Nix is a package manager (and optionally a full OS) built around one idea:
every package is a pure function of its inputs. The same inputs always produce
the same output, on any machine. A *flake* is a self-contained Nix project that
declares its own dependencies in `flake.nix` and locks them to exact hashes in
`flake.lock`, giving you fully reproducible builds.

A `devShell` is a temporary shell that provides a curated set of tools without
touching your normal environment. When you exit the shell those tools disappear.

### Prerequisites

1. [Install Nix](https://nixos.org/download) (single-user or multi-user).
2. Enable flakes and the `nix-command` experimental feature. Add the following
   to `~/.config/nix/nix.conf` (or `/etc/nix/nix.conf`):

   ```
   experimental-features = nix-command flakes
   ```

3. *(Optional but recommended)* Install
   [`direnv`](https://direnv.net/) and
   [`nix-direnv`](https://github.com/nix-community/nix-direnv).
   The `.envrc` file at the root of this repo will then activate the dev shell
   automatically whenever you `cd` into the project directory.

### Entering the dev shell

```bash
nix develop          # enter the shell manually
```

Or, if you use direnv:

```bash
direnv allow         # run once; the shell activates on every cd after that
```

The dev shell provides:

| Tool | Purpose |
|------|---------|
| `cargo` / `rustc` | Compile and run Rust code |
| `rustfmt` | Auto-format source files |
| `clippy` | Lint Rust code |
| `rust-analyzer` | LSP backend for editors |
| `shelltest` | Run the CLI test suite |

### Building and testing inside the dev shell

Once inside `nix develop` (or with direnv active), the standard workflow is:

```bash
# 1. Compile
cargo build

# 2. Run unit and integration tests
cargo test

# 3. Run the CLI (shelltestrunner) tests
#    These require the binary to be built first (step 1).
shelltest tests/cli/

# 4. Check formatting
cargo fmt --check

# 5. Run the linter
cargo clippy -- -D warnings
```

All commands above use the exact tool versions pinned in `flake.lock`, so
results are identical regardless of what is installed on the host system.

The binary accepts two commands:

```
minic --check <file.minic>   # parse + type-check only
minic --run   <file.minic>   # parse + type-check + interpret
```

---

## CLI Usage

### Running a valid program

Save the following as `hello.minic`:

```c
void main() {
  str name = "Alice";
  print(name)
}
```

```bash
$ minic --run hello.minic
Alice
```

### Checking a program without running it

`--check` stops after the type checker. Useful for validating a program before
committing to run it:

```bash
$ minic --check hello.minic
'hello.minic' is well-typed.
```

### Malformed programs

The MiniC parser is lenient by design: it uses a `many0` combinator that
collects as many valid function declarations as it can and stops silently on
anything it cannot recognise. This means most malformed programs do not produce
a parse error — they produce an empty (or partial) function list, and the type
checker then reports the problem.

**Using an unknown keyword instead of a type:**

```c
def greet(str name) {
  print(name)
}
```

The parser skips the unrecognised `def` line and finds no functions. The type
checker then rejects the result:

```bash
$ minic --check bad_keyword.minic
Type error: program must have a main function
```

**Assigning the wrong type to a variable:**

```c
void main() {
  int x = "hello"
}
```

This parses successfully but is rejected by the type checker:

```bash
$ minic --check type_mismatch.minic
Type error: declaration of x: expected Int, got Str
```

### Type errors

Type errors are caught after parsing, before any code runs.

**Assigning a string to an int variable:**

```c
void main() {
  int x = "hello"
}
```

```bash
$ minic --check type_mismatch.minic
Type error: declaration of x: expected Int, got Str
```

**Calling a function with the wrong argument type:**

```c
int double(int n) {
  return n * 2
}

void main() {
  int result = double("hello")
}
```

```bash
$ minic --check wrong_arg.minic
Type error: argument 1 to double: expected Int, got Str
```

**Using a boolean where an integer is expected:**

```c
void main() {
  int x = 10;
  int y = x + true
}
```

```bash
$ minic --check bool_in_arithmetic.minic
Type error: arithmetic operands must be Int or Float
```

### Exit codes

| Situation | Exit code |
|-----------|-----------|
| Success | `0` |
| Parse error, type error, or runtime error | `1` |
| Wrong arguments or missing file | `1` |

---

## Documentation

Read the documents in order for a complete picture of the project, or jump
directly to the topic you need.

| # | File | What you will learn |
|---|------|---------------------|
| 1 | [Language reference](docs/01-language.md) | What you can write in MiniC: types, statements, operators |
| 2 | [Pipeline overview](docs/02-pipeline.md) | How source code travels from text to execution |
| 3 | [The AST](docs/03-ast.md) | How a MiniC program is represented in memory |
| 4 | [The parser](docs/04-parser.md) | How source text is turned into an AST |
| 5 | [The type checker](docs/05-type-checker.md) | How type errors are caught before running |
| 6 | [The interpreter](docs/06-interpreter.md) | How a type-checked program is executed |
| 7 | [The standard library](docs/07-stdlib.md) | Built-in functions and how to add new ones |
| 8 | [Testing](docs/08-testing.md) | How the test suite is organised and how to add tests |

---

## Project Layout

```
src/
├── ir/           # AST node definitions
├── parser/       # Source text → unchecked AST
├── semantic/     # Type checker: unchecked AST → checked AST
├── environment/  # Shared symbol table used by type checker and interpreter
├── interpreter/  # Tree-walking interpreter
└── stdlib/       # Built-in functions (print, sqrt, pow, …)

tests/            # Integration tests (all tests live here)
docs/             # This documentation
```
