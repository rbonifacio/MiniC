## 1. IR module

- [ ] 1.1 Write `//!` comment in `src/ir/mod.rs`: overview of the AST phase, list `ast` sub-module
- [ ] 1.2 Write `//!` comment in `src/ir/ast.rs`: purpose, main types (`Expr`, `Stmt`, `FunDecl`, `Program`, `CheckedProgram`, `Type`), design decisions (plain data nodes, `Checked*` vs unchecked AST split, why the interpreter cannot accept an unchecked program)

## 2. Parser module

- [ ] 2.1 Write `//!` comment in `src/parser/mod.rs`: overview of the parsing phase, list public entry points (`program`, `expression`, `statement`, `fun_decl`, `identifier`, `literal`), design decisions (nom combinator approach vs hand-written recursive descent, sub-module decomposition by syntactic category, why the parser produces an untyped AST)
- [ ] 2.2 Write `//!` comment in `src/parser/expressions.rs`: purpose and key combinator patterns used
- [ ] 2.3 Write `//!` comment in `src/parser/statements.rs`: purpose and statement forms handled
- [ ] 2.4 Write `//!` comment in `src/parser/functions.rs`: purpose and function declaration parsing
- [ ] 2.5 Write `//!` comment in `src/parser/literals.rs`: purpose and literal forms handled
- [ ] 2.6 Write `//!` comment in `src/parser/identifiers.rs`: purpose and identifier rules
- [ ] 2.7 Write `//!` comment in `src/parser/program.rs`: purpose (top-level entry point) and what a valid MiniC program structure looks like

## 3. Semantic module

- [ ] 3.1 Write `//!` comment in `src/semantic/mod.rs`: overview of semantic analysis, public API (`type_check`, `TypeError`), design decisions (separate pass vs integrated with parser, production of `CheckedProgram`)
- [ ] 3.2 Write `//!` comment in `src/semantic/type_checker.rs`: explain use of `Environment<Type>` for variable tracking, how function signatures are resolved, and why `Type::Any` is needed for `print`

## 4. Environment module

- [ ] 4.1 Write `//!` comment in `src/environment/mod.rs`: overview, public API (`Environment<V>`)
- [ ] 4.2 Write `//!` comment in `src/environment/env.rs`: design decisions — parametric `<V>` type parameter (plain-English gloss), single struct serving type checker and interpreter, `snapshot`/`restore` for function call scoping, `names`/`remove_new` for block scoping, alternative (scope stack / linked environments) considered and rejected

## 5. Interpreter module

- [ ] 5.1 Write `//!` comment in `src/interpreter/mod.rs`: overview, public entry point (`interpret`), sub-modules (`eval_expr`, `exec_stmt`, `value`), design decisions — tree-walking approach, why it was chosen over bytecode compilation, functions stored in the same environment as variables
- [ ] 5.2 Write `//!` comment in `src/interpreter/value.rs`: purpose, main types (`Value`, `FnValue`, `RuntimeError`, `NativeFn`), design decisions — `Value` enum as a runtime type (gloss enum-with-data for students), `FnValue` unifying user-defined and native functions, function pointer type `NativeFn` (plain-English gloss)
- [ ] 5.3 Write `//!` comment in `src/interpreter/eval_expr.rs`: purpose (expression evaluation), relationship to `exec_stmt`, note on how function calls are dispatched
- [ ] 5.4 Write `//!` comment in `src/interpreter/exec_stmt.rs`: purpose (statement execution), how it drives `eval_expr`, how block scoping is handled via `names`/`remove_new`

## 6. Stdlib module

- [ ] 6.1 Write `//!` comment in `src/stdlib/mod.rs`: overview, public API (`NativeRegistry`, `NativeEntry`), design decisions — registry pattern, bundling type signature with Rust implementation, why `print` uses `Type::Any`, gloss on function pointer type
- [ ] 6.2 Write `//!` comment in `src/stdlib/io.rs`: purpose, functions exposed (`print_fn`, `read_int_fn`, `read_float_fn`, `read_string_fn`)
- [ ] 6.3 Write `//!` comment in `src/stdlib/math.rs`: purpose, functions exposed (`pow_fn`, `sqrt_fn`), note on numeric coercion via `to_float`

## 7. Verification

- [ ] 7.1 Run `cargo doc --no-deps --open` and visually confirm all six module pages render the two sections correctly
- [ ] 7.2 Run `cargo test` to confirm no source changes broke existing tests
