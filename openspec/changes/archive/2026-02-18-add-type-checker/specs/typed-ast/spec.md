# Typed AST

## Purpose

Single parameterized AST representation for MiniC: expressions and statements with type decoration. Parser produces unchecked (`Ty = ()`); type checker produces checked (`Ty = Type`). Consumed by interpretation or code generation.

## ADDED Requirements

### Requirement: Type representation

The type checker SHALL represent MiniC types as an enum with variants for scalar types, arrays, functions, and unit.

#### Scenario: Scalar types

- **WHEN** a type is represented
- **THEN** it SHALL be one of `Int`, `Float`, `Bool`, or `Str`

#### Scenario: Unit type

- **WHEN** a statement type is represented (imperative statements)
- **THEN** it SHALL be `Unit`

#### Scenario: Array type

- **WHEN** an array type is represented
- **THEN** it SHALL be `Array(Box<Type>)` where the inner type is the element type

#### Scenario: Function type

- **WHEN** a function type is represented
- **THEN** it SHALL be `Fun(Vec<Type>, Box<Type>)` for parameter types and return type

---

### Requirement: Parameterized expression nodes

The AST SHALL use `Expr<Ty>` and `ExprD<Ty>` where `ExprD` wraps an expression with its type decoration.

#### Scenario: ExprD structure

- **WHEN** an expression is represented
- **THEN** it SHALL be `ExprD<Ty> { exp: Expr<Ty>, ty: Ty }` where `Ty = ()` for unchecked and `Ty = Type` for checked

#### Scenario: Expr variants

- **WHEN** the AST has expression variants (Literal, Ident, Add, Index, etc.)
- **THEN** they SHALL contain `ExprD<Ty>` (not raw `Expr`) for subexpressions, so the type parameter propagates through the tree

#### Scenario: Unchecked vs checked

- **WHEN** the parser produces an expression
- **THEN** it SHALL use `ExprD<()>` with `ty: ()` (zero-sized)
- **AND WHEN** the type checker produces an expression
- **THEN** it SHALL use `ExprD<Type>` with `ty: Type`

---

### Requirement: Type synonyms for checked and unchecked

The AST module SHALL define type aliases for checked and unchecked expressions, statements, function declarations, and programs.

#### Scenario: Expression type synonyms

- **WHEN** unchecked or checked expressions are used in code
- **THEN** `UncheckedExpr` SHALL alias `ExprD<()>` and `CheckedExpr` SHALL alias `ExprD<Type>`

#### Scenario: Statement type synonyms

- **WHEN** unchecked or checked statements are used in code
- **THEN** `UncheckedStmt` SHALL alias `StatementD<()>` and `CheckedStmt` SHALL alias `StatementD<Type>`

#### Scenario: Program and function type synonyms

- **WHEN** unchecked or checked programs or function declarations are used in code
- **THEN** `UncheckedProgram` / `CheckedProgram` SHALL alias `Program<()>` / `Program<Type>`
- **AND** `UncheckedFunDecl` / `CheckedFunDecl` SHALL alias `FunDecl<()>` / `FunDecl<Type>`

---

### Requirement: Parameterized statement nodes

The AST SHALL use `Statement<Ty>` and `StatementD<Ty>` where `StatementD` wraps a statement with its type decoration.

#### Scenario: StatementD structure

- **WHEN** a statement is represented
- **THEN** it SHALL be `StatementD<Ty> { stmt: Statement<Ty>, ty: Ty }` where `Ty = ()` for unchecked and `Ty = Type` for checked

#### Scenario: Statement variants

- **WHEN** the AST has statement variants (Assign, Block, Call, If, While)
- **THEN** they SHALL contain `ExprD<Ty>` and `StatementD<Ty>` for subexpressions/substatements

#### Scenario: Statement type

- **WHEN** a statement is type-checked
- **THEN** its `ty` SHALL typically be `Unit` for imperative statements

---

### Requirement: Parameterized program and function nodes

The AST SHALL use `FunDecl<Ty>` and `Program<Ty>` parameterized by the decoration type.

#### Scenario: Program structure

- **WHEN** a program is represented
- **THEN** it SHALL have `functions: Vec<FunDecl<Ty>>` and `body: Vec<StatementD<Ty>>`

#### Scenario: FunDecl structure

- **WHEN** a function declaration is represented
- **THEN** it SHALL have `name`, `params`, `return_type` (annotated), and `body: StatementD<Ty>`
