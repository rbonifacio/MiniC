## ADDED Requirements

### Requirement: Every module has a structured doc comment

Every module entry point (`mod.rs` or single-file module) SHALL have a `//!`
inner doc comment containing two sections: `# Overview` and
`# Design Decisions`. Sub-module files that own a significant design decision
SHALL also carry a `//!` comment for that decision.

#### Scenario: Module without a doc comment
- **WHEN** a module file contains no `//!` comment
- **THEN** the implementation MUST add one before any `use` statements or
  `pub mod` declarations

#### Scenario: Module with only a one-liner comment
- **WHEN** a module file has a `//!` comment shorter than two sentences
- **THEN** the comment MUST be expanded to include both the Overview and
  Design Decisions sections

---

### Requirement: Overview section states purpose and public API

The `# Overview` section SHALL describe, in two to five sentences, what the
module is responsible for and which types or functions it exposes to the rest
of the codebase.

#### Scenario: Overview for interpreter module
- **WHEN** a student reads `src/interpreter/mod.rs`
- **THEN** the Overview MUST name the entry-point function (`interpret`),
  state that it takes a `CheckedProgram` and runs it starting at `main`,
  and list the three sub-modules (`eval_expr`, `exec_stmt`, `value`)

#### Scenario: Overview for a sub-module
- **WHEN** a student reads `src/interpreter/value.rs`
- **THEN** the Overview MUST name the `Value` enum and the `RuntimeError`
  struct and briefly state their role in the interpreter

---

### Requirement: Design Decisions section explains key architectural choices

The `# Design Decisions` section SHALL document every design decision
assigned to that module in the design document (D5 table). Each decision
MUST state what was chosen and why, and MUST mention at least one
alternative that was considered and rejected.

#### Scenario: Tree-walking decision in interpreter
- **WHEN** a student reads `src/interpreter/mod.rs`
- **THEN** the Design Decisions section MUST explain what a tree-walking
  interpreter is, why it was chosen over bytecode compilation, and what
  the trade-off is

#### Scenario: Parametric environment decision
- **WHEN** a student reads `src/environment/env.rs`
- **THEN** the Design Decisions section MUST explain that `Environment<V>`
  is generic (parameterised by type `V`), that this lets the same struct
  serve both the type checker (`V = Type`) and the interpreter (`V = Value`),
  and why duplicating the struct was rejected

#### Scenario: Parser combinator decision
- **WHEN** a student reads `src/parser/mod.rs`
- **THEN** the Design Decisions section MUST explain what the `nom` library
  does, contrast it with hand-writing a recursive-descent parser, and note
  the sub-module decomposition by syntactic category

#### Scenario: Checked vs unchecked AST decision
- **WHEN** a student reads `src/ir/mod.rs`
- **THEN** the Design Decisions section MUST explain the difference between
  the raw AST produced by the parser and the `Checked*` AST produced by the
  type checker, and why this split makes it impossible to interpret an
  unchecked program

#### Scenario: NativeRegistry decision in stdlib
- **WHEN** a student reads `src/stdlib/mod.rs`
- **THEN** the Design Decisions section MUST explain what `NativeRegistry`
  is, why it bundles a type signature alongside the Rust function, and why
  this is needed for type-checking stdlib calls

---

### Requirement: Rust constructs are explained on first appearance per module

Whenever a module introduces a Rust-specific construct that a beginner may
not recognise, the doc comment SHALL include a one-sentence plain-English
gloss of that construct as it is used in this codebase. The gloss MUST NOT
require the student to know Rust terminology in advance.

#### Scenario: Enum with data in value module
- **WHEN** a student reads `src/interpreter/value.rs`
- **THEN** the doc comment MUST explain that `enum Value { Int(i64), … }`
  means "a value can be one of several shapes, each carrying its own data",
  and that Rust calls each shape a *variant*

#### Scenario: Generic type parameter in environment
- **WHEN** a student reads `src/environment/env.rs`
- **THEN** the doc comment MUST explain that the `<V>` in `Environment<V>`
  is a placeholder for any concrete type, allowing the same struct to be
  reused with different value types

#### Scenario: Function pointer type in stdlib
- **WHEN** a student reads `src/stdlib/mod.rs` or `src/interpreter/value.rs`
- **THEN** the doc comment MUST explain that `NativeFn` (defined as
  `fn(Vec<Value>) -> Result<Value, RuntimeError>`) is a function pointer —
  a variable that holds a reference to a Rust function rather than data

---

### Requirement: Doc comments describe current code only

Doc comments SHALL describe the observable behaviour of the code as it
currently exists. They MUST NOT describe planned features, aspirational
designs, or historical states of the code.

#### Scenario: Accurate description of snapshot/restore
- **WHEN** a student reads the doc for `Environment::snapshot` and `restore`
- **THEN** the explanation MUST match the actual mechanism: the entire
  binding map is cloned before a function call and restored after it returns,
  not a scope stack or a linked-environment chain

#### Scenario: No forward-looking statements
- **WHEN** any doc comment is written
- **THEN** it MUST NOT contain phrases like "will be extended", "future work",
  or "planned" unless those plans are tracked in a separate change
