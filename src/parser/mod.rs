//! Parser for the MiniC language.
//!
//! # Overview
//!
//! This module turns a `&str` of MiniC source code into an
//! [`UncheckedProgram`](crate::ir::ast::UncheckedProgram) — a tree of AST
//! nodes where every type decoration is `()` (not yet known).
//!
//! Public entry points:
//!
//! * [`program()`] — parse a complete MiniC program (one or more function
//!   declarations). This is the main entry point used by the pipeline.
//! * [`expression`] — parse a single expression (useful for tests).
//! * [`statement`] / [`assignment`] — parse a single statement.
//! * [`fun_decl`] — parse a single function declaration.
//! * [`identifier`] — parse an identifier.
//! * [`literal`] — parse a literal value.
//!
//! All parsers return an `IResult<&str, T>` from the `nom` library. In
//! simple terms, this is either `Ok((remaining_input, parsed_value))` on
//! success or an error on failure.
//!
//! # Design Decisions
//!
//! ## Parser combinators with `nom`
//!
//! Rather than writing a hand-crafted recursive-descent parser, MiniC uses
//! the [`nom`](https://docs.rs/nom) library. `nom` provides small building
//! blocks — *combinators* — that each recognise a tiny piece of syntax, and
//! which can be composed into larger parsers using functions like `alt`
//! (try alternatives in order), `tuple` (match a sequence), `map`
//! (transform the result), and `preceded` (match A then B, return B).
//!
//! The main advantage is that the structure of the code closely mirrors the
//! grammar of the language, making it easy to add or modify rules. The
//! trade-off is that `nom` error messages can be harder to read than those
//! from a hand-written parser.
//!
//! ## Sub-module decomposition by syntactic category
//!
//! The parser is split into six files, each responsible for one grammatical
//! category: `literals`, `identifiers`, `expressions`, `statements`,
//! `functions`, and `program`. This mirrors how language grammars are
//! traditionally presented and makes it easy to find the rule for any
//! construct. Lower-level modules (e.g., `literals`) are imported by
//! higher-level ones (e.g., `expressions`), so the dependency direction
//! follows the grammar hierarchy.
//!
//! ## The parser produces an untyped AST
//!
//! The parser's only job is to recognise structure — it does not perform any
//! type inference. Every node in the output tree carries `ty: ()`. Type
//! information is filled in by the separate `semantic` module in the next
//! pipeline stage. Keeping parsing and type checking separate makes each
//! phase simpler and easier to test independently.

pub mod expressions;
pub mod functions;
pub mod identifiers;
pub mod literals;
pub mod program;
pub mod statements;
pub mod types;

pub use expressions::expression;
pub use functions::fun_decl;
pub use identifiers::identifier;
pub use literals::{literal, Literal};
pub use program::program;
pub use statements::{assignment, statement};
pub use types::tagged_type_decl;
