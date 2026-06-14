## Why

MiniC has a complete frontend (parser + type checker) but no way to execute programs — the pipeline ends after type checking with no observable output. Adding a tree-walking interpreter over the type-checked AST closes this gap, making MiniC a fully runnable language without requiring code generation.

## What Changes

- Add a `Value` enum representing runtime values (integer, float, boolean, string, array).
- Add an `Interpreter` that walks a `CheckedProgram` and evaluates it.
- Evaluation of all expression forms: literals, arithmetic, comparisons, logical ops, array indexing, function calls.
- Execution of all statement forms: declarations, assignments (including array element assignment), blocks, if/while, return.
- Runtime environment tracking variable bindings (values, not types) and function definitions.
- A top-level `interpret(program: &CheckedProgram)` entry point that locates and executes `main`.
- Built-in `print` function to produce observable output.

## Capabilities

### New Capabilities

- `interpreter-core`: Runtime value representation (`Value` enum), the interpreter struct, environment management for values, and the `interpret()` entry point.
- `expression-eval`: Evaluation of all `Expr` variants from the checked AST to `Value`, including arithmetic coercion (int→float), short-circuit logical operators, array indexing, and function calls.
- `statement-exec`: Execution of all `Statement` variants: variable declarations, assignments to identifiers and array elements, blocks (with scoped environments), if/while control flow, return via a `ReturnSignal` mechanism, and statement-level function calls.
- `function-dispatch`: Function call resolution — looking up function bodies, binding arguments to parameters, executing the body, and unwrapping return values. Includes support for the built-in `print` function.

### Modified Capabilities

_(none — no existing spec requirements change)_

## Impact

- **New module**: `src/interpreter/` (or `src/eval/`) with `mod.rs`, `value.rs`, `env.rs`, `eval_expr.rs`, `exec_stmt.rs`.
- **Existing code**: No changes to the parser, type checker, or AST. The interpreter operates on the existing `CheckedProgram` / `CheckedExpr` types.
- **`lib.rs`**: Export the new interpreter module.
- **`main.rs`**: Wire parser → type checker → interpreter for end-to-end execution.
- **Dependencies**: No new crates required; standard Rust only.
- **Tests**: New integration tests in `tests/interpreter.rs` using `.minic` fixture files, asserting on interpreter output or return values.
