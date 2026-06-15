## ADDED Requirements

### Requirement: Variable declaration execution
The interpreter SHALL execute `Statement::Decl { name, ty, init }` by evaluating `init` to a `Value` and binding `name` to that value in the current `RuntimeEnv`. The declared type `ty` is used for documentation only at runtime (type checking has already validated it).

#### Scenario: Integer declaration
- **WHEN** `int x = 42;` is executed
- **THEN** the environment SHALL contain a binding `x → Value::Int(42)`

#### Scenario: Array declaration
- **WHEN** `int[] arr = [1, 2];` is executed
- **THEN** the environment SHALL contain `arr → Value::Array([Value::Int(1), Value::Int(2)])`

---

### Requirement: Assignment execution
The interpreter SHALL execute `Statement::Assign { target, value }` by evaluating `value` to a `Value` and storing it at the lvalue described by `target`. Supported lvalue forms are:
- `Expr::Ident(name)` — update an existing variable binding in the environment.
- `Expr::Index { base, index }` — update an element of an array in the environment (including nested indexing).
If the target variable does not exist in the environment the interpreter SHALL return a `RuntimeError`.

#### Scenario: Simple variable assignment
- **WHEN** `x = 10;` is executed and `x` is already bound in the environment
- **THEN** the binding for `x` SHALL be updated to `Value::Int(10)`

#### Scenario: Array element assignment
- **WHEN** `arr[0] = 99;` is executed and `arr` is `Value::Array([Value::Int(1), Value::Int(2)])`
- **THEN** `arr` SHALL become `Value::Array([Value::Int(99), Value::Int(2)])`

#### Scenario: Nested array element assignment
- **WHEN** `matrix[1][0] = 5;` is executed and `matrix` is a 2D array
- **THEN** the element at row 1, column 0 SHALL be updated to `Value::Int(5)`

---

### Requirement: Block execution
The interpreter SHALL execute `Statement::Block { seq }` by taking a snapshot of the current environment, executing each statement in `seq` in order, then restoring the snapshot. If any statement produces an early return signal that signal SHALL be propagated immediately and the remaining statements SHALL NOT be executed.

#### Scenario: Block restores scope on exit
- **WHEN** a block declares `int y = 7;` and the block exits normally
- **THEN** `y` SHALL NOT be present in the environment after the block

#### Scenario: Block propagates return signal
- **WHEN** a block contains `return 1;` followed by more statements
- **THEN** execution SHALL stop at `return 1;` and the remaining statements SHALL NOT execute

---

### Requirement: If statement execution
The interpreter SHALL execute `Statement::If { cond, then_branch, else_branch }` by evaluating `cond` to a `Value::Bool`. If `true`, the interpreter SHALL execute `then_branch`; if `false` and `else_branch` is present, the interpreter SHALL execute `else_branch`; otherwise no branch is executed.

#### Scenario: Condition true executes then branch
- **WHEN** `if true then { int x = 1; }` is executed
- **THEN** `x` SHALL be bound in the environment after the then-branch completes

#### Scenario: Condition false with else branch
- **WHEN** `if false then { int x = 1; } else { int x = 2; }` is executed
- **THEN** the else branch SHALL execute and `x` SHALL be bound to `Value::Int(2)` within that scope

#### Scenario: Condition false with no else branch
- **WHEN** `if false then { int x = 1; }` is executed
- **THEN** neither branch executes and the environment is unchanged

---

### Requirement: While statement execution
The interpreter SHALL execute `Statement::While { cond, body }` by repeatedly evaluating `cond` and executing `body` as long as `cond` evaluates to `Value::Bool(true)`. If `cond` is initially `false` the body SHALL NOT execute. If `body` produces an early return signal that signal SHALL terminate the loop and be propagated.

#### Scenario: Loop runs expected number of times
- **WHEN** a while loop with a counter variable runs until the counter reaches 3
- **THEN** the counter SHALL equal `Value::Int(3)` after the loop

#### Scenario: Loop does not execute when condition is initially false
- **WHEN** `while false do { ... }` is executed
- **THEN** the body SHALL NOT execute

---

### Requirement: Return statement execution
The interpreter SHALL execute `Statement::Return(expr)` by evaluating `expr` (if present) and producing an early-return signal carrying the resulting `Value`. For `return;` (no expression) the signal SHALL carry `Value::Void`. The signal SHALL unwind through enclosing blocks up to the function boundary.

#### Scenario: Return with value
- **WHEN** `return 42;` is executed inside a function body
- **THEN** an early-return signal carrying `Value::Int(42)` SHALL be propagated to the function's call site

#### Scenario: Void return
- **WHEN** `return;` is executed inside a void function
- **THEN** an early-return signal carrying `Value::Void` SHALL be propagated

---

### Requirement: Statement-level function call execution
The interpreter SHALL execute `Statement::Call { name, args }` by evaluating all arguments, dispatching to the named function (or built-in), and discarding the return value. Any `RuntimeError` from the call SHALL be propagated.

#### Scenario: Call statement executes and discards return value
- **WHEN** `print(42);` is executed as a statement
- **THEN** the side effect (printing) SHALL occur and no value SHALL be bound
