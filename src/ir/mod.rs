//! Intermediate representation (IR) for MiniC.
//!
//! # Overview
//!
//! This module defines the data structures that represent a MiniC program
//! in memory. It contains a single sub-module, [`ast`], which provides all
//! the types needed to describe expressions, statements, function declarations,
//! and complete programs.
//!
//! The IR is the **common language** between the pipeline stages: the parser
//! produces it, the type checker reads and annotates it, and the interpreter
//! executes it.
//!
//! # Design Decisions
//!
//! ## Plain data, no behaviour
//!
//! AST nodes are plain structs and enums with public fields — they carry data
//! but have no methods that encode language semantics. All logic (type rules,
//! evaluation rules) lives in the `semantic` and `interpreter` modules. This
//! keeps the IR stable and easy to read in isolation.
//!
//! ## A single parameterised AST (the `Ty` decoration)
//!
//! Rather than defining two separate sets of types — one for the parser output
//! and one for the type-checker output — the AST is *parameterised* by a type
//! variable `Ty`. In Rust, this is written as `Expr<Ty>`, meaning "an
//! expression that carries a decoration of type `Ty` at each node".
//!
//! * When `Ty = ()` (the empty tuple, pronounced "unit"), nodes carry no type
//!   information. This is the raw output of the parser.
//! * When `Ty = Type`, every node records the MiniC type the type checker
//!   inferred for it. This is the output of semantic analysis.
//!
//! Type aliases (`UncheckedProgram`, `CheckedProgram`, etc.) give each phase
//! a descriptive name without introducing new types.
//!
//! ## Why this split matters
//!
//! Because the interpreter only accepts a `CheckedProgram`, the Rust compiler
//! statically prevents running a program that has not been type-checked. You
//! cannot accidentally pass raw parser output to the interpreter — the types
//! simply do not match.

pub mod ast;
pub mod tac;
