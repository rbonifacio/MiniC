## 1. Module Setup

- [x] 1.1 Create `src/ir/mod.rs` with `pub mod ast`
- [x] 1.2 Add `mod ir` to `src/main.rs` (or lib.rs)

## 2. Literal and Identifier Nodes

- [x] 2.1 Define `Literal` enum in `ir/ast.rs` with variants `Int(i64)`, `Float(f64)`, `Str(String)`, `Bool(bool)`
- [x] 2.2 Add `Literal` and `Ident(String)` variants to `Expr` enum

## 3. Expression Nodes

- [x] 3.1 Add arithmetic variants to `Expr`: `Neg`, `Add`, `Sub`, `Mul`, `Div` (using `Box<Expr>`)
- [x] 3.2 Add relational variants to `Expr`: `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`
- [x] 3.3 Add boolean variants to `Expr`: `Not`, `And`, `Or`

## 4. Statement Nodes

- [x] 4.1 Define `Stmt` enum with `Assign { target: String, value: Box<Expr> }`
- [x] 4.2 Add `If { cond, then_branch, else_branch: Option<Box<Stmt>> }` variant
- [x] 4.3 Add `While { cond: Box<Expr>, body: Box<Stmt> }` variant

## 5. Program Root

- [x] 5.1 Define `Program` struct with `statements: Vec<Stmt>`

## 6. Derives and Verification

- [x] 6.1 Add `#[derive(Debug, PartialEq)]` to `Literal`, `Expr`, `Stmt`, and `Program`
- [x] 6.2 Run `cargo build` and verify the `ir::ast` module compiles
