## ADDED Requirements

### Requirement: README is a self-contained landing page
`README.md` SHALL contain: a one-paragraph plain-English description of
MiniC, a complete runnable MiniC program (at least a `main` function with
one output), the three essential commands (`cargo build`, `cargo test`,
`cargo run`), and a table listing all eight `docs/` files with one-line
descriptions. It MUST NOT contain implementation detail or architecture
diagrams.

#### Scenario: First-time reader lands on README
- **WHEN** a student opens the project for the first time
- **THEN** they can find a sample MiniC program, run the test suite, and
  know which document to read first — all without leaving `README.md`

#### Scenario: README links to all eight docs
- **WHEN** a student reads the README navigation table
- **THEN** every entry in the table links to a file that exists in `docs/`

---

### Requirement: Language reference document exists
`docs/01-language.md` SHALL describe the MiniC language from a *user*
perspective with no reference to implementation. It MUST cover: all scalar
types (`int`, `float`, `bool`, `str`), array types, variable declaration
syntax, all statement forms (`if`, `while`, block, `return`, call,
assignment), operator precedence as a table, and a complete factorial
program as a worked example.

#### Scenario: Student learns what they can write
- **WHEN** a student reads `01-language.md`
- **THEN** they can write a syntactically valid MiniC program without
  consulting any other document

#### Scenario: Operator precedence is explicit
- **WHEN** a student looks up whether `*` binds tighter than `+`
- **THEN** they find a precedence table in `01-language.md`

---

### Requirement: Pipeline overview document exists
`docs/02-pipeline.md` SHALL describe the five pipeline stages (parser,
unchecked AST, type checker, checked AST, interpreter) using a visual
diagram and one paragraph per stage. It MUST NOT contain Rust code. Each
stage description MUST name its input type and output type in plain English.

#### Scenario: Student understands data flow
- **WHEN** a student reads `02-pipeline.md`
- **THEN** they can describe what each stage receives and produces without
  reading any source code

#### Scenario: Diagram is present
- **WHEN** a student opens `02-pipeline.md`
- **THEN** a text or ASCII diagram showing the five stages and their
  connections is visible near the top of the document

---

### Requirement: AST document explains the Ty decoration accessibly
`docs/03-ast.md` SHALL explain the parameterised AST (`Expr<Ty>`,
`ExprD<Ty>`) and the `Unchecked*` / `Checked*` aliases. It MUST use a
plain-English analogy for the `Ty` parameter. It MUST NOT reference Haskell
GADTs, SmartPy, or any language or framework external to the project.

#### Scenario: Ty parameter explained without prior knowledge
- **WHEN** a student with no Haskell background reads `03-ast.md`
- **THEN** they understand what `Ty = ()` and `Ty = Type` mean and why
  two instantiations exist

#### Scenario: No external references
- **WHEN** a reviewer reads `03-ast.md`
- **THEN** the document contains no references to Haskell, SmartPy, GADTs,
  or any concept requiring knowledge outside this project

---

### Requirement: Parser document leads with a worked example
`docs/04-parser.md` SHALL open with a worked example showing how a short
MiniC expression (e.g., `1 + 2`) is parsed step by step before introducing
`nom` combinators. The combinator introduction MUST explain at least five
`nom` building blocks with one-sentence descriptions. The operator precedence
chain MUST be shown as a diagram.

#### Scenario: Example before concept
- **WHEN** a student reads the first two sections of `04-parser.md`
- **THEN** they encounter a concrete parse trace before the word "combinator"
  is introduced

#### Scenario: Precedence chain is visual
- **WHEN** a student looks up how precedence is implemented
- **THEN** they find a diagram or structured list showing the chain from
  `expression` down to `atom`

---

### Requirement: Type-checker document uses concrete error walkthroughs
`docs/05-type-checker.md` SHALL explain type checking through at least three
concrete MiniC error examples (e.g., undeclared variable, type mismatch,
wrong arity), showing the source program and the error message produced. It
MUST NOT contain a technique comparison table. It MUST NOT reference Haskell,
SmartPy, or any external framework.

#### Scenario: Error walkthrough present
- **WHEN** a student reads `05-type-checker.md`
- **THEN** they find at least three distinct MiniC programs that trigger
  type errors, each with the exact error message shown

#### Scenario: No technique comparison
- **WHEN** a reviewer reads `05-type-checker.md`
- **THEN** the document contains no table comparing implementation
  techniques (A/B/C/D or equivalent)

---

### Requirement: Interpreter document opens with an eval trace
`docs/06-interpreter.md` SHALL open with a step-by-step evaluation trace of
a short arithmetic expression (e.g., `2 + 3 * 4`) showing each recursive
call and intermediate value before explaining the general mechanism.

#### Scenario: Eval trace is the opening example
- **WHEN** a student reads the first substantive section of `06-interpreter.md`
- **THEN** they see a concrete step-by-step trace of expression evaluation
  before any general description of the tree-walking algorithm

---

### Requirement: Stdlib document includes a how-to-add guide
`docs/07-stdlib.md` SHALL include a step-by-step guide for adding a new
native function to MiniC, listing every file that must be changed and the
exact code pattern to follow.

#### Scenario: Student can add a new stdlib function
- **WHEN** a student follows the how-to-add guide in `07-stdlib.md`
- **THEN** they can implement and register a new native function with no
  other documentation needed

---

### Requirement: Testing document covers all five test files
`docs/08-testing.md` SHALL describe the purpose and structure of each of the
five integration test files (`parser.rs`, `type_checker.rs`, `interpreter.rs`,
`stdlib.rs`, `program.rs`). It MUST include a worked example of writing one
new test from scratch. It MUST reflect that stdlib tests now live in
`tests/stdlib.rs` (not in `src/`).

#### Scenario: All test files are documented
- **WHEN** a student reads `08-testing.md`
- **THEN** they find a description of all five test files including
  `tests/stdlib.rs`

#### Scenario: New test walkthrough present
- **WHEN** a student wants to add a test for a new feature
- **THEN** `08-testing.md` contains a step-by-step example they can follow

---

### Requirement: Old doc/ directory is removed
The `doc/` directory (containing `summary.md`, `architecture/`, and
`design/`) SHALL be deleted once all content has been migrated to `docs/`.
No file in `doc/` SHALL remain after the change is complete.

#### Scenario: doc/ directory no longer exists
- **WHEN** the change is complete and the repository is inspected
- **THEN** there is no `doc/` directory at the project root

#### Scenario: All content is preserved or intentionally removed
- **WHEN** a reviewer compares old and new docs
- **THEN** every topic covered in `doc/` is either present in `docs/` or
  has been explicitly dropped (e.g., the technique comparison table) with
  a documented reason

---

### Requirement: Documents are numbered and cross-linked
Each document in `docs/` SHALL begin with a line indicating its position in
the reading sequence and end with a "What to read next" line linking to the
following document. `docs/08-testing.md` SHALL link back to `README.md`.

#### Scenario: Reading order is navigable
- **WHEN** a student finishes reading any document in `docs/`
- **THEN** they can click a link at the bottom to proceed to the next one
  without consulting `README.md`
