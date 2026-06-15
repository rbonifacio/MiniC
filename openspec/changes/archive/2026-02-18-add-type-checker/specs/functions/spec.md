## MODIFIED Requirements

### Requirement: Parse function declarations

The parser SHALL recognize function declarations of the form `def identifier ( param_list ) -> return_type statement`. The parser SHALL produce `ir::ast::FunDecl` with name, parameter list, return type, and body.

#### Scenario: Function with parameters and return type

- **WHEN** the input is `def foo(x, y) -> Int x = x + y`
- **THEN** the parser SHALL succeed and return `FunDecl { name: "foo", params: ["x", "y"], return_type: Int, body }`

#### Scenario: Function with no parameters

- **WHEN** the input is `def bar() -> Unit x = 1`
- **THEN** the parser SHALL succeed with `params: []` and the declared return type

#### Scenario: Optional whitespace

- **WHEN** the input is `def  foo  ( x , y )  ->  Int  x = 1`
- **THEN** the parser SHALL succeed

#### Scenario: Return type annotation required

- **WHEN** a function declaration is parsed
- **THEN** the return type annotation (`-> Type`) SHALL be required
- **AND** the parser SHALL produce `FunDecl` with a `return_type` field
