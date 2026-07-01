# Identifiers

## Purpose

MiniC identifiers (variable names).

## Requirements

### Requirement: Parse identifiers

The parser SHALL recognize identifiers (variable names). An identifier MUST start with a letter (a-z, A-Z) or underscore (`_`). Subsequent characters MAY be letters, digits (0-9), or underscores. The parser SHALL return the identifier as a string slice or owned string.

#### Scenario: Simple identifier

- **WHEN** the input is `x` or `count` or `_temp`
- **THEN** the parser SHALL succeed and return the identifier

#### Scenario: Identifier with digits

- **WHEN** the input is `var1` or `max_value_42`
- **THEN** the parser SHALL succeed and return the identifier

#### Scenario: Reject identifier starting with digit

- **WHEN** the input is `1var` or `42`
- **THEN** the parser SHALL fail (return `Err`)

#### Scenario: Reject reserved words as identifiers

- **WHEN** the input is `true` or `false`
- **THEN** the parser SHALL fail when parsing as identifier (these are parsed as boolean literals in expression context)
