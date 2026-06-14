# Type Checker Design

## Context

MiniC parses source into an AST (`ir::ast`) with no type information. The AST has literals (Int, Float, Str, Bool), identifiers, arithmetic/relational/boolean operators, function calls, arrays, and indexed assignment. A semantic analyzer must validate types and produce a typed AST for downstream phases (interpretation, code generation). The design should achieve Haskell GADT–style guarantees: unchecked AST in, checked typed AST out (or errors).

## Goals / Non-Goals

**Goals:**

- Type-check the AST and report all type errors
- Produce a typed AST with `Type` attached to each expression/statement node
- Int/float coercion: mixed numeric expressions promote to float (e.g., `1 + 3.14` → float)
- Phase separation: unchecked and checked AST are distinct types; downstream phases consume only the typed AST

**Non-Goals:**

- Type inference beyond literal and coercion (identifiers get types from context)
- Optimization or constant folding

## Single AST with Decorated Fields (SmartPy-Style)

In Haskell (e.g., SmartPy's `Syntax.hs`), a single AST representation is used for both unchecked and checked phases. The trick is **`Deco s t`**:

```haskell
data Switch = Unchecked | Checked

data Deco (s :: Switch) t where
  U :: Deco 'Unchecked t      -- constructor with no argument
  T :: t -> Deco 'Checked t   -- constructor with type value

data ExprD (s :: Switch) = ExprD
  { exp       :: Expr s,
    expType   :: Deco s Type   -- U when unchecked, T ty when checked
  }
```

- **Unchecked**: `expType` is `U` (no data, zero-sized).
- **Checked**: `expType` is `T ty` (carries the type).
- **Single structure**: `Expr s`, `ExprD s`, `StatementD s` are parameterized by `s`; the same constructors, but the decor field's *type* changes with `s`.

### Can Rust Simulate This?

**Yes.** Rust can achieve the same effect using a **generic type parameter** for the decoration:

```rust
// Option A: Generic over decoration type directly
struct ExprD<Ty> {
    exp: Expr<Ty>,
    ty: Ty,   // Ty = () for unchecked (zero-sized), Ty = Type for checked
}

enum Expr<Ty> {
    Literal(Literal),
    Add(Box<ExprD<Ty>>, Box<ExprD<Ty>>),
    // ...
}
```

- `Expr<()>` / `ExprD<()>`: unchecked; `ty: ()` is zero-sized.
- `Expr<Type>` / `ExprD<Type>`: checked; `ty: Type` carries the type.

**Option B: Trait + associated type** (closer to Haskell's kind-level `Switch`):

```rust
trait Phase {
    type TypeDecor;
}
impl Phase for Unchecked {
    type TypeDecor = ();
}
impl Phase for Checked {
    type TypeDecor = Type;
}

struct ExprD<P: Phase> {
    exp: Expr<P>,
    ty: P::TypeDecor,
}
```

Both give compile-time phase separation: `ExprD<Checked>` and `ExprD<Unchecked>` are different types; codegen cannot accept unchecked AST.

### Trade-offs vs. Two Parallel ASTs

| Aspect | Single AST (Deco-style) | Two Parallel ASTs |
|--------|-------------------------|-------------------|
| **Code duplication** | None; one definition | Duplicate `Expr` / `TypedExpr` |
| **Parser changes** | Parser must produce `Expr<()>` / `ExprD<()>` | Parser unchanged; produces current `Expr` |
| **Conversion** | Parser output needs `ExprD` wrapper with `ty: ()` | Type checker builds `TypedExpr` from scratch |
| **Recursion** | `Expr<Ty>` contains `ExprD<Ty>`; mutual recursion | Straightforward; no mutual recursion |
| **Pattern matching** | One set of `match` arms; `ty` present in both phases | Two sets; different types to match |
| **Complexity** | Higher: generic everywhere, `Expr`/`ExprD` split | Lower: familiar, incremental |

**Parser impact**: The current MiniC parser returns `Expr` (no type slot). For the single-AST approach, we must either (a) change the parser to produce `ExprD<()>` with `ty: ()` at each node, or (b) add a conversion pass that wraps `Expr` → `ExprD<()>`. Both are feasible.

### Conclusion

Rust *can* simulate the SmartPy `Deco` pattern with a single parameterized AST. We adopt this approach for MiniC.

---

## Decisions

### 1. Single AST with Deco-Style Generics (Adopted)

**Choice:** One parameterized AST: `Expr<Ty>`, `ExprD<Ty>`, `Statement<Ty>`, `StatementD<Ty>`, `FunDecl<Ty>`, `Program<Ty>`. `Ty = ()` for unchecked, `Ty = Type` for checked. Both expressions and statements carry type decoration.

**Rationale:** Single representation; no duplication. Compile-time phase separation: `Program<Type>` vs `Program<()>` are distinct types. Matches SmartPy/Haskell design.

**Structure:**
- `ExprD<Ty>`: `{ exp: Expr<Ty>, ty: Ty }` — expressions with type decor
- `StatementD<Ty>`: `{ stmt: Statement<Ty>, ty: Ty }` — statements with type decor (typically `Unit` for imperative stmts)
- Parser produces `ExprD<()>`, `StatementD<()>`, etc. with `ty: ()` at each node.

**Type synonyms:** The implementation SHALL define type aliases for checked and unchecked variants:

| Alias | Definition | Use |
|-------|------------|-----|
| `UncheckedExpr` | `ExprD<()>` | Parser output |
| `CheckedExpr` | `ExprD<Type>` | Type checker output |
| `UncheckedStmt` | `StatementD<()>` | Parser output |
| `CheckedStmt` | `StatementD<Type>` | Type checker output |
| `UncheckedFunDecl` | `FunDecl<()>` | Parser output |
| `CheckedFunDecl` | `FunDecl<Type>` | Type checker output |
| `UncheckedProgram` | `Program<()>` | Parser output |
| `CheckedProgram` | `Program<Type>` | Type checker output |

These aliases improve readability and document intent at call sites.

### 2. Parser Returns ExprD / StatementD

**Choice:** The parser SHALL produce `ExprD<()>` and `StatementD<()>` directly (with `ty: ()` at each node), not raw `Expr` / `Stmt`.

**Rationale:** No conversion pass; the AST is parameterized from the start.

### 3. Int/Float Coercion Rules

**Choice:** When both operands are numeric, promote to the wider type:

| Left   | Right | Result |
|--------|-------|--------|
| Int    | Int   | Int    |
| Int    | Float | Float  |
| Float  | Int   | Float  |
| Float  | Float | Float  |

Applies to `+`, `-`, `*`, `/`. Relational operators use same coercion; result is `Bool`. Boolean operators require `Bool` operands.

**Rationale:** Matches common language semantics (C, Python). Avoids surprising failures for `1 + 3.14`.

### 4. Type Representation

**Choice:** `enum Type { Unit, Int, Float, Bool, Str, Array(Box<Type>), Fun(Vec<Type>, Box<Type>) }`.

**Rationale:** Covers current AST. `Unit` for statement types (imperative stmts). `Array` recursive for nested arrays. `Fun` for function types (params → return).

### 5. Error Reporting

**Choice:** Stop at the first error. Return `Result<Program<Type>, TypeError>` (single error).

**Rationale:** Simpler implementation; fail fast.

### 6. Scope Rules

**Choice:** Function-local scope. Each function has its own scope; parameters are in scope for the body. Global scope for the program body (variables assigned before use).

**Rationale:** Matches typical C-like semantics.

### 7. Function Return Type Annotations

**Choice:** Function declarations SHALL be annotated with return type information. The type checker validates that the body conforms to the declared return type.

**Rationale:** Explicit signatures; enables type-checking calls against declarations.

## Risks / Trade-offs

- **[Parser refactor]** Parser must produce `ExprD<()>`, `StatementD<()>` → Mitigation: Change AST and parser in one change; all expression/statement construction sites add `ty: ()`.
- **[Identifier typing]** Identifiers need a symbol table; undeclared vars are errors → Mitigation: Build env per function and for global body.
- **[Function syntax]** Return type annotations require grammar change → Mitigation: Add syntax for `def name(params) -> ReturnType body` (or similar).

## Migration Plan

1. Refactor AST: introduce `Expr<Ty>`, `ExprD<Ty>`, `Statement<Ty>`, `StatementD<Ty>`, `FunDecl<Ty>`, `Program<Ty>` with `Ty = ()` for unchecked.
2. Add `Type` enum including `Unit`, `Fun`.
3. Update parser to produce `ExprD<()>`, `StatementD<()>` (add `ty: ()` at each node).
4. Add grammar for function return type annotations.
5. Add `src/semantic/mod.rs` and `src/semantic/type_checker.rs`.
6. Implement `type_check(program: &Program<()>) -> Result<Program<Type>, TypeError>`.
7. Wire into main: parse → type_check → (future: interpret).

## Open Questions

- Exact syntax for function return type (e.g., `def foo(x) -> Int x = 1` vs `def foo(x): Int x = 1`).
