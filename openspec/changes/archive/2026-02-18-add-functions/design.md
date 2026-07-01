## Context

MiniC currently has no functions. We need declarations (define a function) and calls (invoke a function). Calls can appear in expressions (`x = foo(1) + 2`) and as statements (`foo(1, 2)` for side effects).

## Goals / Non-Goals

**Goals:**

- Parse function declarations: `def name(param1, param2, ...) body`
- Parse function calls as expressions: `name(arg1, arg2, ...)`
- Parse function calls as statements: `name(arg1, arg2)` (standalone)
- Update Program to include function declarations
- Update parser documentation

**Non-Goals:**

- Return values, return statements (functions can "return" via future work)
- Semantic analysis (e.g., arity checking)
- Built-in functions

## Decisions

### 1. Declaration syntax: `def name(params) body`

**Choice:** Use `def` keyword, then identifier, then `(param1, param2, ...)`, then a single statement as body. No braces; body is one statement (use `if` or `while` for blocks).

**Rationale:** Consistent with `if expr then stmt` and `while expr do stmt`; minimal syntax.

### 2. Call as expression: add to `primary`

**Choice:** In `primary`, add `identifier ( expr_list )` as an alternative. Try call (identifier followed by `(`) before plain identifier to avoid ambiguity. Order: literal | call | identifier | parenthesized.

**Rationale:** `foo(1)` must parse as call, not `Ident("foo")` then `(1)` as invalid. So we need call before identifier in `primary`. But `primary` currently has identifier for variables. So we need: if we see identifier and then `(`, it's a call; otherwise it's a variable. The parser can try `identifier` + `(` + `arglist` + `)` first, and only if that fails, try plain `identifier`. Actually the cleanest is: parse `identifier` then look for `(`. So we could have a combined parser: `identifier` optionally followed by `( arglist )`. If `(` is present, it's a call; otherwise it's Ident. So `primary` = literal | identifier_or_call | parenthesized.

### 3. Call as statement

**Choice:** Add call as a statement: `identifier ( expr_list )`. In `statement`, try `call_statement` (identifier + `(` + arglist + `)`) before assignment. Order: if | while | call | assignment.

**Rationale:** Keyword-based (if, while) first; then call (has `(`); then assignment (has `=`). No ambiguity.

### 4. AST: `FunDecl` and `Expr::Call`

**Choice:** Add `FunDecl { name: String, params: Vec<String>, body: Box<Stmt> }`. Add `Expr::Call { name: String, args: Vec<Expr> }`. No separate `Stmt::Call`; use `Stmt::Expr(Expr::Call(...))` or add `Stmt::Call` for clarity. Simpler: add `Stmt::Call { name: String, args: Vec<Expr> }` to avoid wrapping.

**Rationale:** Explicit `Stmt::Call` makes the AST clearer; expression-statement could be `Stmt::Expr(Box<Expr>)` but that would allow any expression as statement (e.g. `1 + 2`). Restricting to call-only for statements keeps it minimal.

### 5. Program structure

**Choice:** `Program { functions: Vec<FunDecl>, body: Vec<Stmt> }`. Functions come first, then the main body statements.

**Rationale:** Common pattern; allows forward reference if we add declaration-before-use later.

### 6. Argument list parsing

**Choice:** Zero or more expressions separated by commas: `( )` or `( expr )` or `( expr , expr , ... )`. Use `separated_list0` or manual loop.

**Rationale:** Standard comma-separated list; trailing comma optional (we can reject for simplicity).
