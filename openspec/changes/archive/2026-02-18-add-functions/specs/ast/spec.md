## MODIFIED Requirements

### Requirement: Statement nodes

Add function call as a statement variant.

#### Scenario: Call statement

- **WHEN** a function call is used as a standalone statement
- **THEN** it SHALL be represented as `Stmt::Call { name: String, args: Vec<Expr> }`

---

### Requirement: Function declarations

The AST SHALL include a node for function declarations.

#### Scenario: Function declaration

- **WHEN** a function is declared
- **THEN** it SHALL be represented as `FunDecl { name: String, params: Vec<String>, body: Box<Stmt> }`

---

### Requirement: Expression nodes

Add function call as an expression variant.

#### Scenario: Call expression

- **WHEN** a function is invoked in expression context
- **THEN** it SHALL be represented as `Expr::Call { name: String, args: Vec<Expr> }`

---

### Requirement: Program root

#### Scenario: Program with functions

- **WHEN** a program includes function declarations
- **THEN** the root SHALL have a `functions: Vec<FunDecl>` field and a `body: Vec<Stmt>` (or equivalent) for the main statements
