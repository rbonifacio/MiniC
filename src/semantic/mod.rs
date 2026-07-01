//! Semantic analysis for MiniC: type checking.
//!
//! # Overview
//!
//! This module is the second stage of the MiniC pipeline. It takes the raw
//! [`UncheckedProgram`](crate::ir::ast::UncheckedProgram) produced by the
//! parser and, if the program is well-typed, returns a
//! [`CheckedProgram`](crate::ir::ast::CheckedProgram) where every AST node
//! is annotated with its inferred MiniC type.
//!
//! Public API (re-exported from `type_checker`):
//!
//! * [`type_check`] — the main entry point. Returns `Ok(CheckedProgram)` or
//!   `Err(TypeError)` on the first type error found.
//! * [`TypeError`] — the error type, carrying a human-readable message.
//!
//! # Design Decisions
//!
//! ## Type checking as a separate pass
//!
//! The type checker is a distinct pipeline stage rather than being woven into
//! the parser. This separation keeps each phase smaller and easier to test:
//! the parser is tested with syntactic inputs, the type checker with
//! well-formed ASTs. It also means the interpreter can accept only a
//! `CheckedProgram`, making it impossible at the Rust type level to run an
//! unchecked program.
//!
//! ## Fail-fast: stop at the first error
//!
//! The current implementation reports only the first type error it
//! encounters. This simplifies the implementation (no need to collect or
//! de-duplicate errors) and is appropriate for a teaching language where
//! programs are short and students fix one error at a time.

pub mod type_checker;

pub use type_checker::{type_check, TypeError};
