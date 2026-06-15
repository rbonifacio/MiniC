## ADDED Requirements

### Requirement: Parse integer literals

The parser SHALL recognize integer literals as sequences of decimal digits, optionally prefixed by a minus sign. Leading zeros are allowed. The parser SHALL return the value as a signed 64-bit integer.

#### Scenario: Positive integer

- **WHEN** the input is `42` or `0` or `12345`
- **THEN** the parser SHALL succeed and return the corresponding integer value

#### Scenario: Negative integer

- **WHEN** the input is `-17` or `-0`
- **THEN** the parser SHALL succeed and return the corresponding negative integer value

#### Scenario: Reject non-integer input

- **WHEN** the input is `abc` or `12.34` or empty
- **THEN** the parser SHALL fail (return `Err`)

---

### Requirement: Parse float literals

The parser SHALL recognize float literals in decimal notation: optional minus, digits, optional decimal point and fractional digits. The parser SHALL return the value as a 64-bit float.

#### Scenario: Float with decimal point

- **WHEN** the input is `3.14` or `0.5` or `-0.25`
- **THEN** the parser SHALL succeed and return the corresponding float value

#### Scenario: Integer-looking float

- **WHEN** the input is `1.` or `.5` (if supported by grammar)
- **THEN** the parser SHALL succeed and return the corresponding float value, or the grammar MAY reject `.5` if not in scope

#### Scenario: Reject non-float input

- **WHEN** the input is `abc` or `"3.14"` or empty
- **THEN** the parser SHALL fail (return `Err`)

---

### Requirement: Parse string literals

The parser SHALL recognize string literals enclosed in double quotes. The parser SHALL support escape sequences `\"`, `\\`, `\n`, and `\t`. The parser SHALL return the decoded string value (without the surrounding quotes).

#### Scenario: Simple string

- **WHEN** the input is `"hello"` or `""`
- **THEN** the parser SHALL succeed and return `hello` or the empty string respectively

#### Scenario: String with escapes

- **WHEN** the input is `"a\"b"` or `"line1\nline2"` or `"tab\there"`
- **THEN** the parser SHALL succeed and return the string with escapes decoded

#### Scenario: Reject unclosed or invalid string

- **WHEN** the input is `"unclosed` or `noquotes` or empty
- **THEN** the parser SHALL fail (return `Err`)

---

### Requirement: Parse boolean literals

The parser SHALL recognize the keywords `true` and `false` as boolean literals. The parser SHALL return the corresponding boolean value.

#### Scenario: True literal

- **WHEN** the input is `true`
- **THEN** the parser SHALL succeed and return `true`

#### Scenario: False literal

- **WHEN** the input is `false`
- **THEN** the parser SHALL succeed and return `false`

#### Scenario: Reject non-boolean input

- **WHEN** the input is `True` or `1` or `maybe` or empty
- **THEN** the parser SHALL fail (return `Err`)
