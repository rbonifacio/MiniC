# Pointers

## Purpose

Document MiniC **pointer syntax** and **intended usage**: pointer types (`T*`),
address-of (`&expr`), dereference (`*expr`), assignment through a dereference
(`*expr = …`), and pointer-typed parameters and return values.

## Requirements

### Requirement: Pointer type notation

The parser SHALL recognise pointer types written as a scalar type name
immediately followed by `*`, with no space between them: `int*`, `float*`,
`bool*`, `str*`. Function parameters, return types, and local declarations
SHALL use this same spelling (as in the three fixture programs).

#### Scenario: Declaration with pointer type

- **WHEN** the source contains `int* y = &x` or `float* a_ref = &a`
- **THEN** the parser SHALL succeed and associate the declared name with
  `Type::Pointer` to the corresponding scalar type

#### Scenario: Function signature with pointers

- **WHEN** the source contains `void increment(int* p)` or
  `int* changeRef(int* x, int* y)`
- **THEN** the parser SHALL succeed with each pointer parameter typed as
  `Pointer` to the inner scalar type

---

### Requirement: Address-of expression

The parser SHALL recognise the unary prefix `&` applied to an expression,
producing `Expr::AddrOf`. The fixtures use `&` only on simple identifiers
(`&x`, `&a`, …), which is the intended teaching subset.

#### Scenario: Initialisation from address of variable

- **WHEN** the input is `int* y = &x` or `bool* d_ref = &d`
- **THEN** the parser SHALL succeed with the initialiser as `AddrOf` wrapping
  the identifier expression

#### Scenario: Address passed to a function

- **WHEN** the input is `increment(&x)` as in `pointer_feature.minic`
- **THEN** the parser SHALL succeed with the call argument as `AddrOf`

---

### Requirement: Dereference expression

The parser SHALL recognise the unary prefix `*` as dereference (same lexical
token as multiplication, resolved in the unary layer), producing `Expr::Deref`.

#### Scenario: Dereference on the right-hand side

- **WHEN** the input is `*p + 1` as in `pointer_feature.minic`
- **THEN** the parser SHALL succeed with `Deref` applied to the operand of `+`
  as appropriate for unary-before-additive precedence

#### Scenario: Nested dereference in assignment target

- **WHEN** the input is `*p = *p + 1`
- **THEN** the parser SHALL succeed with assignment whose target expression is
  `Deref` and whose value expression uses `Deref` on the same pointer

---

### Requirement: Assignment to a dereference

The parser SHALL accept assignment statements whose target is a dereference
expression `*expr`, as in `*p = *p + 1`.

#### Scenario: Mutate through pointer parameter

- **WHEN** the statement is `*p = *p + 1` inside `increment` in
  `pointer_feature.minic`
- **THEN** the parser SHALL produce `Stmt::Assign` with a `Deref` target

---

### Requirement: Return type and return value as pointer

The parser SHALL allow a function to declare a pointer return type and
`return` an expression of pointer type, as in `changeRef` in
`pointer_function.minic`.

#### Scenario: Return pointer from function

- **WHEN** the function is `int* changeRef(int* x, int* y) { … return x; }`
- **THEN** the parser SHALL succeed with return type `Pointer(Int)` and a
  return statement carrying the pointer expression

---

### Requirement: Assignment between pointer variables

The parser SHALL allow assignment where both sides are pointer-typed
expressions (e.g. `x = y` when `x` and `y` are `int*`), as in the body of
`changeRef` in `pointer_function.minic`.

#### Scenario: Rebind pointer parameter

- **WHEN** the statement is `x = y` with `x` and `y` declared as `int*`
  parameters
- **THEN** the parser SHALL succeed with `Stmt::Assign` and identifier
  expressions for `x` and `y`