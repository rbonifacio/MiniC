---
name: "miniC educational parser code"
description: Coding standards for this repo
---

When working in this project, prioritize code that is:

- clear and easy to read for educational purposes
- self-documenting through explicit naming, structure, and straightforward control flow
- idiomatic Rust, but not at the expense of readability
- aligned with the existing miniC codebase style and parser-combinator design

Prefer:

- descriptive function, type, and variable names
- simple parser structure and well-scoped helper functions
- comments only when they explain why a design choice matters, not what obvious code does
- preserving spec-driven behavior and making language rules understandable

Avoid:

- overly terse or clever code that reduces comprehension
- large unrelated refactors when the task is focused on parser/AST/spec behavior
- introducing new patterns that conflict with the established codebase style

IMPORTANT! Always thoroughly review the relevant `docs/` documentation before starting a task and again whenever you encounter a roadblock.
