## 1. Setup

- [ ] 1.1 Create the `docs/` directory at the project root
- [ ] 1.2 Read all existing files in `doc/` to extract content worth preserving before deleting anything

## 2. README

- [ ] 2.1 Rewrite `README.md`: one-paragraph description of MiniC, a complete sample program, `cargo build` / `cargo test` / `cargo run` commands, and a navigation table linking to all eight `docs/` files

## 3. docs/01-language.md

- [ ] 3.1 Write the language overview: all scalar types with examples, array syntax, variable declaration
- [ ] 3.2 Write all statement forms with short examples (declaration, assignment, if/else, while, block, return, call)
- [ ] 3.3 Add operator precedence table (from lowest to highest)
- [ ] 3.4 Add a complete factorial program as the closing worked example
- [ ] 3.5 Add "What to read next → 02-pipeline.md" footer

## 4. docs/02-pipeline.md

- [ ] 4.1 Write the five-stage ASCII/text pipeline diagram (source → parser → unchecked AST → type checker → checked AST → interpreter)
- [ ] 4.2 Write one paragraph per stage: name, input type, output type, one-sentence purpose — no Rust code
- [ ] 4.3 Add "What to read next → 03-ast.md" footer

## 5. docs/03-ast.md

- [ ] 5.1 Introduce the AST concept with a plain-English analogy for the `Ty` parameter (e.g., "a sticky note on each node")
- [ ] 5.2 Explain `Ty = ()` (unchecked, parser output) vs `Ty = Type` (checked, type checker output) with short code snippets
- [ ] 5.3 List and describe the main node types: `Expr`, `ExprD`, `Statement`, `StatementD`, `FunDecl`, `Program`
- [ ] 5.4 Explain the `Unchecked*` / `Checked*` type aliases and why they make the compiler enforce phase ordering
- [ ] 5.5 Verify no external references (Haskell, SmartPy, GADTs) remain
- [ ] 5.6 Add "What to read next → 04-parser.md" footer

## 6. docs/04-parser.md

- [ ] 6.1 Write the opening worked example: trace how `1 + 2` is parsed step by step
- [ ] 6.2 Introduce `nom` combinators with at least five building blocks and one-sentence descriptions each
- [ ] 6.3 Add the sub-module decomposition section (literals → identifiers → expressions → statements → functions → program)
- [ ] 6.4 Add the operator precedence chain as a diagram (expression → logical_or → … → atom)
- [ ] 6.5 Explain left-associativity via the accumulator loop pattern
- [ ] 6.6 Add "What to read next → 05-type-checker.md" footer

## 7. docs/05-type-checker.md

- [ ] 7.1 Write three concrete MiniC error examples with source program and exact error message (undeclared variable, type mismatch, wrong arity)
- [ ] 7.2 Explain `Environment<Type>` and how variable and function types are tracked
- [ ] 7.3 Explain the `fn_snapshot` technique for mutual recursion support
- [ ] 7.4 Explain block scoping via snapshot/restore
- [ ] 7.5 Explain `Type::Any` and why `print` uses it
- [ ] 7.6 Verify no technique comparison table (A/B/C/D) remains
- [ ] 7.7 Verify no Haskell/SmartPy references remain
- [ ] 7.8 Add "What to read next → 06-interpreter.md" footer

## 8. docs/06-interpreter.md

- [ ] 8.1 Write the opening eval trace: step-by-step evaluation of `2 + 3 * 4` showing each recursive call and intermediate value
- [ ] 8.2 Explain the `Value` enum variants with one-line descriptions and examples
- [ ] 8.3 Explain `FnValue` (UserDefined vs Native) and how call dispatch works
- [ ] 8.4 Explain `ExecResult = Option<Value>` and how early return propagates
- [ ] 8.5 Explain block scoping (names/remove_new) and function call scoping (snapshot/restore)
- [ ] 8.6 Add "What to read next → 07-stdlib.md" footer

## 9. docs/07-stdlib.md

- [ ] 9.1 Document each built-in function from a user perspective: name, signature, description, example call
- [ ] 9.2 Explain `NativeRegistry` and `NativeEntry` (type signature + function pointer)
- [ ] 9.3 Explain why `print` uses `Type::Any`
- [ ] 9.4 Write the step-by-step guide for adding a new native function (every file to edit, exact code pattern)
- [ ] 9.5 Add "What to read next → 08-testing.md" footer

## 10. docs/08-testing.md

- [ ] 10.1 Describe the purpose and structure of each of the five test files: `parser.rs`, `type_checker.rs`, `interpreter.rs`, `stdlib.rs`, `program.rs`
- [ ] 10.2 Explain when to use inline string tests vs fixture files
- [ ] 10.3 Write a worked example: adding one new test end to end (choose a simple interpreter test)
- [ ] 10.4 Verify `tests/stdlib.rs` is documented (not `src/stdlib/`)
- [ ] 10.5 Add "What to read next → README.md" footer

## 11. Cross-links and cleanup

- [ ] 11.1 Verify all links in `README.md` point to existing `docs/` files
- [ ] 11.2 Verify each document's "What to read next" link resolves correctly
- [ ] 11.3 Delete `doc/summary.md`
- [ ] 11.4 Delete `doc/architecture/` and all its contents
- [ ] 11.5 Delete `doc/design/` and all its contents
- [ ] 11.6 Delete the now-empty `doc/` directory
