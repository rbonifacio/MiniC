# Tasks: Add Type Checker

## 1. AST refactor (parameterized types)

- [x] 1.1 Add `Type` enum to `src/ir/ast.rs` (Unit, Int, Float, Bool, Str, Array(Box<Type>), Fun(Vec<Type>, Box<Type>))
- [x] 1.2 Introduce `ExprD<Ty>` and `Expr<Ty>`; change `Expr` variants to use `ExprD<Ty>` for subexpressions
- [x] 1.3 Introduce `StatementD<Ty>` and `Statement<Ty>`; change `Statement` variants to use `ExprD<Ty>` and `StatementD<Ty>`
- [x] 1.4 Add `return_type: Type` to `FunDecl<Ty>`; parameterize `FunDecl` and `Program` by `Ty`
- [x] 1.5 Ensure `ExprD<()>` and `StatementD<()>` compile with `ty: ()` (zero-sized)
- [x] 1.6 Add type synonyms: `UncheckedExpr`/`CheckedExpr`, `UncheckedStmt`/`CheckedStmt`, `UncheckedProgram`/`CheckedProgram`, `UncheckedFunDecl`/`CheckedFunDecl`

## 2. Parser updates

- [x] 2.1 Add grammar for function return type: `def name(params) -> Type body`
- [x] 2.2 Update expression parser to produce `ExprD<()>` with `ty: ()` at each node
- [x] 2.3 Update statement parser to produce `StatementD<()>` with `ty: ()` at each node
- [x] 2.4 Update function parser to parse return type and produce `FunDecl<()>`
- [x] 2.5 Update all `map`/construction sites to wrap in `ExprD { exp, ty: () }` / `StatementD { stmt, ty: () }`

## 3. Type checker module

- [x] 3.1 Create `src/semantic/mod.rs` and `src/semantic/type_checker.rs`
- [x] 3.2 Define `TypeError` struct/enum with error kind and location/context
- [x] 3.3 Implement `type_check(program: &Program<()>) -> Result<Program<Type>, TypeError>` entry point (single error, fail-fast)
- [x] 3.4 Add `semantic` module to `lib.rs`

## 4. Expression type checking

- [x] 4.1 Type-check literals (Int, Float, Str, Bool → corresponding Type)
- [x] 4.2 Type-check identifiers (lookup from scope; error if undeclared)
- [x] 4.3 Type-check arithmetic (+, -, *, /) with int/float coercion
- [x] 4.4 Type-check unary minus (Neg) with int/float
- [x] 4.5 Type-check relational operators (==, !=, <, <=, >, >=) with coercion, result Bool
- [x] 4.6 Type-check boolean operators (and, or, !) — operands Bool, result Bool
- [x] 4.7 Type-check array literals (elements same type after coercion → Array(elem_ty))
- [x] 4.8 Type-check index expressions (base: Array(t), index: Int → t)
- [x] 4.9 Type-check function calls (arg count/types match declaration; return type)

## 5. Statement and program type checking

- [x] 5.1 Type-check assignment (target and value compatible; update scope for identifier targets)
- [x] 5.2 Type-check block (sequential statements, scope inheritance)
- [x] 5.3 Type-check call statement
- [x] 5.4 Type-check if/while (cond: Bool, bodies recursively type-checked)
- [x] 5.5 Type-check function declarations (params in scope for body; body type matches return_type)
- [x] 5.6 Type-check program (functions then body; fail at first error)

## 6. Scope and environment

- [x] 6.1 Implement symbol table / environment for identifier → Type
- [x] 6.2 Function-local scope: params in scope for function body
- [x] 6.3 Global scope for program body
- [x] 6.4 Handle assignment targets (identifiers, index expressions) — update scope

## 7. Tests and integration

- [x] 7.1 Add tests for literal typing
- [x] 7.2 Add tests for int/float coercion (int+float, float+int, etc.)
- [x] 7.3 Add tests for type errors (undeclared var, type mismatch, invalid index) — verify fail-fast
- [ ] 7.4 Add tests for array typing (literal, index, indexed assignment)
- [ ] 7.5 Add tests for function return type validation
- [ ] 7.6 Add tests for function-local scope
- [ ] 7.7 Wire type_check into main or add example that parses and type-checks

## 8. Documentation

- [x] 8.1 Add `doc/architecture/ast.md` documenting the checked vs unchecked AST design (see design in openspec)
