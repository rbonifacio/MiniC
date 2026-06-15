## ADDED Requirements

### Requirement: Check command
The binary SHALL accept `--check <file>` as arguments, parse the given source file, and run the type checker. If both succeed, the binary SHALL print a success message to stdout and exit with code 0. If either step fails, the binary SHALL print a diagnostic message to stderr and exit with code 1.

#### Scenario: Valid program passes check
- **WHEN** the user runs `minic --check valid.minic` and the file parses and type-checks without errors
- **THEN** the binary prints a success confirmation to stdout and exits with code 0

#### Scenario: Parse error on check
- **WHEN** the user runs `minic --check bad.minic` and the file contains a syntax error
- **THEN** the binary prints a parse error message to stderr and exits with code 1

#### Scenario: Type error on check
- **WHEN** the user runs `minic --check bad.minic` and the file has a type error
- **THEN** the binary prints a type error message to stderr and exits with code 1

### Requirement: Run command
The binary SHALL accept `--run <file>` as arguments, parse the given source file, run the type checker, and then interpret the program. If all steps succeed the binary SHALL exit with code 0. If any step fails, the binary SHALL print a diagnostic message to stderr and exit with code 1.

#### Scenario: Valid program runs successfully
- **WHEN** the user runs `minic --run valid.minic` and the program parses, type-checks, and executes without errors
- **THEN** the binary exits with code 0

#### Scenario: Parse error on run
- **WHEN** the user runs `minic --run bad.minic` and the file contains a syntax error
- **THEN** the binary prints a parse error message to stderr and exits with code 1, without attempting type-checking or interpretation

#### Scenario: Type error on run
- **WHEN** the user runs `minic --run bad.minic` and the file has a type error
- **THEN** the binary prints a type error message to stderr and exits with code 1, without attempting interpretation

#### Scenario: Runtime error on run
- **WHEN** the user runs `minic --run bad.minic` and the program raises a runtime error during interpretation
- **THEN** the binary prints the runtime error message to stderr and exits with code 1

### Requirement: Usage on invalid invocation
The binary SHALL print a usage message to stderr and exit with code 1 whenever it is invoked with no arguments, an unrecognized flag, or a flag without a following file path argument.

#### Scenario: No arguments provided
- **WHEN** the user runs `minic` with no arguments
- **THEN** the binary prints a usage message to stderr and exits with code 1

#### Scenario: Unknown flag provided
- **WHEN** the user runs `minic --unknown file.minic`
- **THEN** the binary prints a usage message to stderr and exits with code 1

#### Scenario: Flag with no file argument
- **WHEN** the user runs `minic --check` with no following file path
- **THEN** the binary prints a usage message to stderr and exits with code 1

### Requirement: Missing file error
The binary SHALL report a clear error and exit with code 1 when the specified source file cannot be read (e.g., does not exist or is not accessible).

#### Scenario: File does not exist
- **WHEN** the user runs `minic --run nonexistent.minic` and the file is not found
- **THEN** the binary prints a file-not-found error message to stderr and exits with code 1
