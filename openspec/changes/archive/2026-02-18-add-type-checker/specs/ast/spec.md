## MODIFIED Requirements

### Requirement: Expression nodes

The AST SHALL include expression nodes for arithmetic, relational, and boolean operations. Expressions SHALL be parameterized by a type decoration `Ty` and SHALL use `ExprD<Ty>` wrappers.

#### Scenario: Parameterized expression structure

- **WHEN** an expression is represented
- **THEN** it SHALL be `ExprD<Ty> { exp: Expr<Ty>, ty: Ty }` where `Ty = ()` for unchecked (parser output) and `Ty = Type` for checked (type checker output)

#### Scenario: Arithmetic operators

- **WHEN** an arithmetic expression is represented
- **THEN** it SHALL support binary operators `+`, `-`, `*`, `/` and unary `-`
- **AND** subexpressions SHALL be `ExprD<Ty>` (e.g., `Add(Box<ExprD<Ty>>, Box<ExprD<Ty>>)`)

#### Scenario: Relational operators

- **WHEN** a relational expression is represented
- **THEN** it SHALL support `==`, `!=`, `<`, `>`, `<=`, `>=`
- **AND** subexpressions SHALL be `ExprD<Ty>`

#### Scenario: Boolean operators

- **WHEN** a boolean expression is represented
- **THEN** it SHALL support `and`, `or`, and unary `!`
- **AND** subexpressions SHALL be `ExprD<Ty>`

#### Scenario: Expression recursion

- **WHEN** an expression contains sub-expressions (e.g., `a + b * c`)
- **THEN** those sub-expressions SHALL be represented as nested `ExprD<Ty>` nodes

#### Scenario: Array literal

- **WHEN** an array literal is parsed
- **THEN** it SHALL be represented as `Expr::ArrayLit(Vec<ExprD<Ty>>)`
- **AND** the vector SHALL contain the element expressions in order (zero or more)

#### Scenario: Index expression

- **WHEN** an index expression is parsed (e.g., `arr[i]`)
- **THEN** it SHALL be represented as `Expr::Index { base: Box<ExprD<Ty>>, index: Box<ExprD<Ty>> }`
- **AND** `base` SHALL be the indexed expression and `index` SHALL be the index expression

#### Scenario: Call expression

- **WHEN** a function is invoked in expression context
- **THEN** it SHALL be represented as `Expr::Call { name: String, args: Vec<ExprD<Ty>> }`

---

### Requirement: Statement nodes

The AST SHALL include statement nodes for assignments, conditionals, loops, function calls, and blocks. Statements SHALL be parameterized by a type decoration `Ty` and SHALL use `StatementD<Ty>` wrappers.

#### Scenario: Parameterized statement structure

- **WHEN** a statement is represented
- **THEN** it SHALL be `StatementD<Ty> { stmt: Statement<Ty>, ty: Ty }` where `Ty = ()` for unchecked and `Ty = Type` for checked

#### Scenario: Assignment statement

- **WHEN** an assignment is represented
- **THEN** it SHALL have a target and a value, both as `ExprD<Ty>`

#### Scenario: Assign with lvalue target

- **WHEN** an assignment is parsed
- **THEN** it SHALL be represented as `Statement::Assign { target: Box<ExprD<Ty>>, value: Box<ExprD<Ty>> }`
- **AND** `target` SHALL be an lvalue: `Expr::Ident` or `Expr::Index` (possibly nested)

#### Scenario: Call statement

- **WHEN** a function call is used as a standalone statement
- **THEN** it SHALL be represented as `Statement::Call { name: String, args: Vec<ExprD<Ty>> }`

#### Scenario: Block statement

- **WHEN** a block of statements is parsed
- **THEN** it SHALL be represented as `Statement::Block { seq: Vec<StatementD<Ty>> }`
- **AND** `seq` SHALL contain the statements in order (zero or more)

#### Scenario: If-then-else statement

- **WHEN** a conditional is represented
- **THEN** it SHALL have a condition `ExprD<Ty>`, a then-branch `StatementD<Ty>`, and an optional else-branch `StatementD<Ty>`

#### Scenario: While statement

- **WHEN** a loop is represented
- **THEN** it SHALL have a condition `ExprD<Ty>` and a body `StatementD<Ty>`

#### Scenario: Statement recursion

- **WHEN** a statement contains other statements (e.g., body of if/while)
- **THEN** those SHALL be represented as nested `StatementD<Ty>` nodes

---

### Requirement: Function declarations

The AST SHALL include a node for function declarations with return type annotation.

#### Scenario: Function declaration

- **WHEN** a function is declared
- **THEN** it SHALL be represented as `FunDecl<Ty> { name: String, params: Vec<String>, return_type: Type, body: Box<StatementD<Ty>> }`
- **AND** `return_type` SHALL be the declared return type from the grammar

---

### Requirement: Program root

The AST SHALL define a root node representing a complete MiniC program.

#### Scenario: Program structure

- **WHEN** a full program is represented
- **THEN** the root SHALL be parameterized: `Program<Ty>`

#### Scenario: Program with functions

- **WHEN** a program includes function declarations
- **THEN** the root SHALL have `functions: Vec<FunDecl<Ty>>` and `body: Vec<StatementD<Ty>>` for the main statements

## ADDED Requirements

### Requirement: Parser output

The parser SHALL produce the unchecked AST with `Ty = ()` at every node.

#### Scenario: Parser produces ExprD

- **WHEN** the parser produces an expression
- **THEN** it SHALL return `ExprD<()>` with `ty: ()` at each expression node

#### Scenario: Parser produces StatementD

- **WHEN** the parser produces a statement
- **THEN** it SHALL return `StatementD<()>` with `ty: ()` at each statement node
