# Add Type Checker (Semantic Analyzer)

## Why

MiniC currently parses source code into an AST but does not validate types or attach type information. A type checker is needed to catch type errors before interpretation or code generation, and to produce a typed AST that downstream phases can rely on. Int/float coercion (e.g., `1 + 3.14` → float) should follow expected semantics.

## What Changes

- Refactor AST to a single parameterized representation: `Expr<Ty>`, `ExprD<Ty>`, `Statement<Ty>`, `StatementD<Ty>`, `FunDecl<Ty>`, `Program<Ty>` with `Ty = ()` for unchecked and `Ty = Type` for checked
- Parser SHALL produce `ExprD<()>` and `StatementD<()>` directly (with `ty: ()` at each node)
- Add function return type annotations to the grammar
- Add a semantic analysis phase that consumes `Program<()>` and produces either a single `TypeError` or `Program<Type>`
- Implement int/float coercion: mixed numeric expressions promote to float
- Add a `Type` enum (`Unit`, `Int`, `Float`, `Bool`, `Str`, `Array`, `Fun`)
- Add type synonyms: `UncheckedExpr`/`CheckedExpr` (= `ExprD<()>`/`ExprD<Type>`), and similarly for statements, programs, function declarations
- Add a `semantic` module with the type checker
- Function-local scope; parameters in scope for function body

## Capabilities

### New Capabilities

- `type-checker`: Semantic analysis that type-checks the AST, reports a single type error (fail-fast), and produces `Program<Type>` with int/float coercion, function return type validation, and function-local scope
- `typed-ast`: Single parameterized AST representation (`ExprD<Ty>`, `StatementD<Ty>`) with type decoration; parser produces unchecked (`Ty = ()`), type checker produces checked (`Ty = Type`)

### Modified Capabilities

- `ast`: AST SHALL be parameterized by type decoration; expressions and statements SHALL use `ExprD<Ty>` and `StatementD<Ty>` wrappers with `ty: Ty`; parser SHALL produce `ExprD<()>`, `StatementD<()>`
- `functions`: Function declarations SHALL include return type annotations; grammar extended for `def name(params) -> ReturnType body`

## Impact

- **AST refactor**: `Expr` → `Expr<Ty>` / `ExprD<Ty>`; `Stmt` → `Statement<Ty>` / `StatementD<Ty>`; `FunDecl`, `Program` parameterized by `Ty`
- **Parser changes**: All expression/statement construction sites add `ty: ()`; produce `ExprD<()>`, `StatementD<()>`
- **New modules**: `src/semantic/` (type checker)
- **Grammar**: Function return type syntax
- **Pipeline**: Parse → TypeCheck → (future: interpret/codegen). Type checker returns `Result<Program<Type>, TypeError>` (single error, fail-fast).
