## ADDED Requirements

### Requirement: Parse for statements

The parser SHALL recognize for statements of the form
`for '(' [init] ';' [expression] ';' [update] ')' block`, where *init* (when
present) is a declaration (`type name = expression`) or an assignment
(`lvalue = expression`) without a trailing `;`, *update* (when present) is an
assignment without a trailing `;`, and *block* is a block statement. The
parser SHALL produce `ir::ast::Statement::For` with optional `init`, `cond`,
`update`, and a mandatory `body` field.

#### Scenario: Simple for with declaration init

- **WHEN** the input is `for (int i = 0; i < 10; i = i + 1) { sum = sum + i; }`
- **THEN** the parser SHALL succeed and return
  `Statement::For { init, cond, update, body }` where `init` is a `Decl`,
  `cond` is a relational `Lt`, `update` is an `Assign`, and `body` is a
  `Block` containing a single assignment.

#### Scenario: For with assignment init

- **WHEN** the input is `for (i = 0; i < n; i = i + 1) { x = x + 1; }`
- **THEN** the parser SHALL succeed and `init` SHALL be a `Statement::Assign`.

#### Scenario: Nested for

- **WHEN** the input is
  `for (int i = 0; i < 2; i = i + 1) { for (int j = 0; j < 2; j = j + 1) { x = x + 1; } }`
- **THEN** the parser SHALL succeed with nested `Statement::For` nodes.

#### Scenario: For inside a function body

- **WHEN** the input is
  `void main() { int sum = 0; for (int i = 0; i < 10; i = i + 1) { sum = sum + i; } print(sum); }`
- **THEN** the function declaration parser SHALL succeed and the block body
  SHALL contain three statements, the middle one being a `Statement::For`.

#### Scenario: Optional whitespace

- **WHEN** the input is
  `for(int i=0;i<10;i=i+1){sum=sum+i;}` or
  `for  (  int  i  =  0  ;  i  <  10  ;  i  =  i  +  1  )  {  sum  =  sum  +  i  ;  }`
- **THEN** the parser SHALL succeed.

#### Scenario: All three header clauses omitted

- **WHEN** the input is `for (;;) { x = x + 1; }`
- **THEN** the parser SHALL succeed and return `Statement::For` where
  `init == None`, `cond == None`, and `update == None`.

#### Scenario: Omitted init clause

- **WHEN** the input is `for (; i < 10; i = i + 1) { x = x + 1; }`
- **THEN** the parser SHALL succeed with `init == None`, `cond == Some(_)`,
  and `update == Some(_)`.

#### Scenario: Omitted condition clause

- **WHEN** the input is `for (i = 0; ; i = i + 1) { x = x + 1; }`
- **THEN** the parser SHALL succeed with `init == Some(_)`, `cond == None`,
  and `update == Some(_)`.

#### Scenario: Omitted update clause

- **WHEN** the input is `for (i = 0; i < 10; ) { x = x + 1; }`
- **THEN** the parser SHALL succeed with `init == Some(_)`, `cond == Some(_)`,
  and `update == None`.

#### Scenario: Reject missing parentheses

- **WHEN** the input is `for int i = 0; i < 10; i = i + 1 { x = 1; }`
- **THEN** the parser SHALL fail.

#### Scenario: Reject missing semicolons in header

- **WHEN** the input is `for (int i = 0 i < 10 i = i + 1) { x = 1; }`
- **THEN** the parser SHALL fail.

#### Scenario: Reject bare body

- **WHEN** the input is `for (int i = 0; i < 10; i = i + 1) sum = sum + i;`
- **THEN** the parser SHALL fail because the body MUST be a block.

#### Scenario: Reject non-assignment update

- **WHEN** the input is `for (int i = 0; i < 10; i + 1) { x = 1; }`
- **THEN** the parser SHALL fail.

#### Scenario: Reject `for` as identifier

- **WHEN** the identifier parser receives the input `for`
- **THEN** it SHALL fail (the keyword is reserved).
