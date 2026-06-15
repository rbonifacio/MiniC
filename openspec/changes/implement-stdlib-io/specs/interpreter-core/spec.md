## MODIFIED Requirements

### Requirement: Interpret entry point
The interpreter SHALL expose a public `interpret(program: &CheckedProgram, registry: &NativeRegistry) -> Result<(), RuntimeError>` function that locates the `main` function in the program, executes it using the provided registry for built-in dispatch, and returns `Ok(())` on success or a `RuntimeError` on failure.

#### Scenario: Successful execution of a minimal program
- **WHEN** `interpret` is called with a `CheckedProgram` containing a `void main()` function with an empty body and a default `NativeRegistry`
- **THEN** the function SHALL return `Ok(())`

#### Scenario: Missing main function
- **WHEN** `interpret` is called with a `CheckedProgram` that contains no function named `main`
- **THEN** the function SHALL return `Err(RuntimeError)` describing the missing entry point

#### Scenario: Built-in called during execution
- **WHEN** `interpret` is called with a program that calls `sqrt(4.0)` and a registry containing `sqrt`
- **THEN** the call SHALL dispatch correctly and the function SHALL return `Ok(())`
