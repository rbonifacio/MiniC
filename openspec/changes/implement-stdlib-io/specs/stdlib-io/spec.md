## ADDED Requirements

### Requirement: print built-in via registry
The stdlib IO module SHALL register `print` in the `NativeRegistry` with signature `(any) → void`. `print` SHALL accept exactly one argument of any `Value` type, format it using `Value`'s `Display` implementation, write it to standard output followed by a newline, and return `Value::Void`.

#### Scenario: Print integer
- **WHEN** `print(42)` is called
- **THEN** `"42\n"` SHALL be written to stdout and `Value::Void` SHALL be returned

#### Scenario: Print boolean
- **WHEN** `print(true)` is called
- **THEN** `"true\n"` SHALL be written to stdout

#### Scenario: Print float
- **WHEN** `print(3.14)` is called
- **THEN** the float's string representation followed by `"\n"` SHALL be written to stdout

#### Scenario: Print string
- **WHEN** `print("hello")` is called
- **THEN** `"hello\n"` SHALL be written to stdout

#### Scenario: Print array
- **WHEN** `print([1, 2, 3])` is called
- **THEN** a human-readable representation of the array (e.g., `"[1, 2, 3]\n"`) SHALL be written to stdout

---

### Requirement: readInt built-in
The stdlib IO module SHALL register `readInt` in the `NativeRegistry` with signature `() → int`. `readInt` SHALL read one line from standard input, trim leading and trailing whitespace, parse it as an `i64`, and return `Value::Int`. If parsing fails, `readInt` SHALL return `Err(RuntimeError)`.

#### Scenario: Valid integer input
- **WHEN** `readInt()` is called and the user enters `"42\n"` on stdin
- **THEN** the result SHALL be `Value::Int(42)`

#### Scenario: Invalid integer input
- **WHEN** `readInt()` is called and stdin contains `"abc\n"`
- **THEN** the result SHALL be `Err(RuntimeError)` with a message describing the parse failure

#### Scenario: Integer input with surrounding whitespace
- **WHEN** `readInt()` is called and stdin contains `"  7  \n"`
- **THEN** the result SHALL be `Value::Int(7)`

---

### Requirement: readFloat built-in
The stdlib IO module SHALL register `readFloat` in the `NativeRegistry` with signature `() → float`. `readFloat` SHALL read one line from standard input, trim whitespace, parse it as an `f64`, and return `Value::Float`. If parsing fails, `readFloat` SHALL return `Err(RuntimeError)`.

#### Scenario: Valid float input
- **WHEN** `readFloat()` is called and stdin contains `"3.14\n"`
- **THEN** the result SHALL be `Value::Float(3.14)`

#### Scenario: Integer-formatted float input
- **WHEN** `readFloat()` is called and stdin contains `"2\n"`
- **THEN** the result SHALL be `Value::Float(2.0)`

#### Scenario: Invalid float input
- **WHEN** `readFloat()` is called and stdin contains `"hello\n"`
- **THEN** the result SHALL be `Err(RuntimeError)`

---

### Requirement: readString built-in
The stdlib IO module SHALL register `readString` in the `NativeRegistry` with signature `() → str`. `readString` SHALL read one line from standard input, trim leading and trailing whitespace, and return `Value::Str`. EOF without input SHALL return `Err(RuntimeError)`.

#### Scenario: Valid string input
- **WHEN** `readString()` is called and stdin contains `"hello world\n"`
- **THEN** the result SHALL be `Value::Str("hello world".to_string())`

#### Scenario: Whitespace is trimmed
- **WHEN** `readString()` is called and stdin contains `"  hi  \n"`
- **THEN** the result SHALL be `Value::Str("hi".to_string())`

#### Scenario: EOF produces error
- **WHEN** `readString()` is called and stdin is closed (EOF)
- **THEN** the result SHALL be `Err(RuntimeError)`
