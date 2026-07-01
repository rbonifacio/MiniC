# Unified Environment

## Purpose

Defines the single parametric `Environment<V>` struct that stores all name-to-value bindings — both variables and functions — in one map. Used for type checking (`V = Type`) and interpretation (`V = Value`).

## Requirements

### Requirement: Unified parametric environment
The system SHALL provide a single `Environment<V>` struct that stores all name-to-value bindings in one `HashMap<String, V>`. Both variable bindings and function bindings SHALL be stored in the same map. `Environment<V>` SHALL be used for type checking (with `V = Type`) and for interpretation (with `V = Value`). The separate `function_signatures`, `function_declarations`, and `fns` maps SHALL be removed.

#### Scenario: Variable binding lookup
- **WHEN** a variable `x` is declared via `env.declare("x", v)`
- **THEN** `env.get("x")` SHALL return `Some(&v)`

#### Scenario: Function binding lookup
- **WHEN** a function `foo` is registered via `env.declare("foo", Value::Fn(...))`
- **THEN** `env.get("foo")` SHALL return `Some` with the function value

#### Scenario: Unknown name returns None
- **WHEN** `env.get("unknown")` is called and no binding exists for that name
- **THEN** it SHALL return `None`

---

### Requirement: Environment scoping operations
`Environment<V>` SHALL support four scoping primitives: `snapshot()` captures a clone of all current bindings, `restore(snapshot)` replaces all bindings with the snapshot, `names()` returns the set of currently bound names, and `remove_new(outer_names)` removes any binding whose name is not in `outer_names`.

#### Scenario: Block scoping via remove_new
- **WHEN** `names()` is called before a block, new variables are declared inside, and `remove_new(outer_names)` is called on block exit
- **THEN** only names present before the block SHALL remain in the environment; function bindings (registered before the block) SHALL be preserved

#### Scenario: Function call scoping via snapshot/restore
- **WHEN** a snapshot is taken before a function call, parameters are bound, the body executes, and `restore(snapshot)` is called after
- **THEN** all bindings SHALL revert to the pre-call state, including function bindings being re-instated unchanged

---

### Requirement: Type::Any for polymorphic parameters
The type system SHALL include a `Type::Any` variant. `types_compatible(t, Type::Any)` SHALL return `true` for any type `t`. `Type::Any` SHALL only appear as a declared parameter type in native function signatures; the parser SHALL NOT produce it and the type checker SHALL NOT infer it as an expression type.

#### Scenario: Any matches every argument type
- **WHEN** a native function declares a parameter of type `Type::Any` and is called with an argument of type `Int`, `Bool`, `Str`, `Float`, or `Array`
- **THEN** the type checker SHALL accept the call without a type error

#### Scenario: Any is not produced by the parser
- **WHEN** any valid MiniC source program is parsed
- **THEN** no AST node SHALL carry `Type::Any`

---

### Requirement: Value::Fn for runtime function representation
The `Value` enum SHALL include a `Value::Fn(FnValue)` variant. `FnValue` SHALL be an enum with two variants: `UserDefined(CheckedFunDecl)` for functions declared in the MiniC source, and `Native(NativeFn)` for Rust-implemented stdlib functions. Both kinds of function SHALL be stored and looked up as `Value::Fn` bindings in `Environment<Value>`.

#### Scenario: User-defined function stored as Value::Fn
- **WHEN** a MiniC function declaration is registered in the runtime environment
- **THEN** it SHALL be stored as `Value::Fn(FnValue::UserDefined(decl))`

#### Scenario: Native function stored as Value::Fn
- **WHEN** a native stdlib function is registered in the runtime environment
- **THEN** it SHALL be stored as `Value::Fn(FnValue::Native(f))`

#### Scenario: Dispatch via unified lookup
- **WHEN** `eval_call` looks up a callee name
- **THEN** it SHALL call `env.get(name)` and dispatch by matching on `Value::Fn(FnValue::UserDefined(...))` or `Value::Fn(FnValue::Native(...))`
