## ADDED Requirements

### Requirement: NativeFn type
The stdlib module SHALL define a `NativeFn` type alias as a plain Rust function pointer:
`type NativeFn = fn(Vec<Value>) -> Result<Value, RuntimeError>`.
All built-in functions SHALL conform to this signature.

#### Scenario: Native function invoked with arguments
- **WHEN** a `NativeFn` is called with a `Vec<Value>` of arguments
- **THEN** it SHALL return either `Ok(Value)` on success or `Err(RuntimeError)` on failure

---

### Requirement: NativeEntry holds signature and implementation together
The stdlib module SHALL define a `NativeEntry` struct containing:
- `params: Vec<Type>` — the MiniC parameter types
- `return_type: Type` — the MiniC return type
- `func: NativeFn` — the Rust implementation

#### Scenario: Entry exposes both type info and callable
- **WHEN** a `NativeEntry` is constructed for a built-in function
- **THEN** `params` and `return_type` SHALL be readable by the type checker and `func` SHALL be callable by the interpreter

---

### Requirement: NativeRegistry maps names to entries
The stdlib module SHALL define a `NativeRegistry` struct that maps function names (`String`) to `NativeEntry` values. The registry SHALL expose:
- `register(name, entry)` to add a built-in
- `lookup(name) -> Option<&NativeEntry>` to retrieve an entry by name
- `NativeRegistry::default()` to construct a registry pre-populated with all stdlib functions

#### Scenario: Lookup of registered function
- **WHEN** a function named `"sqrt"` is registered and `registry.lookup("sqrt")` is called
- **THEN** the returned `NativeEntry` SHALL have `params = [Type::Float]` and `return_type = Type::Float`

#### Scenario: Lookup of unregistered function
- **WHEN** `registry.lookup("unknown")` is called
- **THEN** the result SHALL be `None`

#### Scenario: Default registry contains all stdlib functions
- **WHEN** `NativeRegistry::default()` is called
- **THEN** the registry SHALL contain entries for `print`, `readInt`, `readFloat`, `readString`, `pow`, and `sqrt`

---

### Requirement: Registry is passed explicitly to type checker and interpreter
The `type_check` function SHALL accept a `&NativeRegistry` parameter alongside the program. The `interpret` function SHALL accept a `&NativeRegistry` parameter alongside the program. Neither function SHALL use a global or thread-local registry.

#### Scenario: Type checker uses registry for built-in signatures
- **WHEN** `type_check` is called with a program that calls `sqrt(4.0)` and a registry containing `sqrt`
- **THEN** type checking SHALL succeed and the call SHALL resolve to return type `Float`

#### Scenario: Interpreter uses registry for built-in dispatch
- **WHEN** `interpret` is called with a program that calls `sqrt(4.0)` and a registry containing `sqrt`
- **THEN** the call SHALL dispatch to the registered Rust implementation and return `Value::Float(2.0)`
