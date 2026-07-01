## ADDED Requirements

### Requirement: AST module location

The AST SHALL be defined in `src/ir/ast.rs`. The `ir` module SHALL represent intermediate representations produced by the parser and consumed by later phases (semantic analysis, code generation).

#### Scenario: Module exists and compiles

- **WHEN** the project is built with `cargo build`
- **THEN** the `ir::ast` module SHALL compile without errors and SHALL be reachable from the crate root

#### Scenario: AST is independent of parser

- **WHEN** the AST types are used by the parser
- **THEN** the parser SHALL depend on `ir::ast`, and `ir::ast` SHALL NOT depend on the parser module

---

### Requirement: Literal and identifier nodes

The AST SHALL include nodes for literals (integer, float, string, boolean) and identifiers (variable names).

#### Scenario: Literal variants

- **WHEN** a literal is represented in the AST
- **THEN** it SHALL be one of `Int(i64)`, `Float(f64)`, `Str(String)`, or `Bool(bool)`

#### Scenario: Identifier representation

- **WHEN** a variable reference is represented in the AST
- **THEN** it SHALL store the identifier as a `String` (or equivalent owned type)

---

### Requirement: Expression nodes

The AST SHALL include expression nodes for arithmetic, relational, and boolean operations.

#### Scenario: Arithmetic operators

- **WHEN** an arithmetic expression is represented
- **THEN** it SHALL support binary operators `+`, `-`, `*`, `/` and unary `-`

#### Scenario: Relational operators

- **WHEN** a relational expression is represented
- **THEN** it SHALL support `==`, `!=`, `<`, `>`, `<=`, `>=`

#### Scenario: Boolean operators

- **WHEN** a boolean expression is represented
- **THEN** it SHALL support `and`, `or`, and unary `not`

#### Scenario: Expression recursion

- **WHEN** an expression contains sub-expressions (e.g., `a + b * c`)
- **THEN** those sub-expressions SHALL be represented as nested expression nodes

---

### Requirement: Statement nodes

The AST SHALL include statement nodes for assignments, conditionals, and loops.

#### Scenario: Assignment statement

- **WHEN** an assignment is represented
- **THEN** it SHALL have an identifier (target) and an expression (value)

#### Scenario: If-then-else statement

- **WHEN** a conditional is represented
- **THEN** it SHALL have a condition expression, a then-branch statement, and an optional else-branch statement

#### Scenario: While statement

- **WHEN** a loop is represented
- **THEN** it SHALL have a condition expression and a body statement

#### Scenario: Statement recursion

- **WHEN** a statement contains other statements (e.g., body of if/while)
- **THEN** those SHALL be represented as nested statement nodes

---

### Requirement: Program root

The AST SHALL define a root node representing a complete MiniC program.

#### Scenario: Program structure

- **WHEN** a full program is represented
- **THEN** the root SHALL be a sequence (or list) of statements

---

### Requirement: Debug and display

AST types SHALL derive `Debug` and `PartialEq` for debugging and testing.

#### Scenario: Debug output

- **WHEN** an AST node is printed with `{:?}` or `dbg!()`
- **THEN** it SHALL produce readable output showing structure and values

#### Scenario: Equality comparison

- **WHEN** two AST nodes are compared with `==`
- **THEN** structural equality SHALL be supported for tests
