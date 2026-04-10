//! Top-level program parser for MiniC.
//!
//! # Overview
//!
//! Exposes one public function:
//!
//! * [`program`] — parses a complete MiniC program as a sequence of zero or
//!   more function declarations and returns an
//!   [`UncheckedProgram`].
//!
//! A valid MiniC program contains **only** function declarations at the top
//! level — there are no top-level statements or variable declarations outside
//! of functions. This constraint is enforced here by the grammar: `program`
//! is defined as `many0(fun_decl)`, so any token that does not start a
//! function declaration causes the parse to stop. The type checker then
//! verifies that a `main` function exists.
//!
//! # Design Decisions
//!
//! ## `many0` as the top-level combinator
//!
//! `nom`'s `many0` combinator repeatedly applies a parser until it fails,
//! collecting results in a `Vec`. Using it here means the program parser
//! naturally handles empty programs (zero functions) and programs with any
//! number of functions with no extra branching logic. The existence of
//! `main` is a semantic constraint checked in the next pipeline stage, not
//! a syntactic one enforced here.

use crate::ir::ast::{Program, TaggedTypeDecl, UncheckedProgram};
use crate::parser::functions::fun_decl;
use crate::parser::types::tagged_type_decl;
use nom::{branch::alt, combinator::map, multi::many0, IResult};

/// Parse a complete MiniC program: zero or more struct or function declarations.
/// Execution starts at the `main` function (validated by the type checker).
pub fn program(input: &str) -> IResult<&str, UncheckedProgram> {
    let (rest, items) = many0(alt((
        map(tagged_type_decl, |decl| Item::TypeDecl(decl)),
        map(fun_decl, |f| Item::Function(f)),
    )))(input)?;

    let mut type_decls = Vec::new();
    let mut functions = Vec::new();
    for item in items {
        match item {
            Item::TypeDecl(decl) => type_decls.push(decl),
            Item::Function(f) => functions.push(f),
        }
    }

    Ok((
        rest,
        Program {
            tagged_types: type_decls,
            functions,
        },
    ))
}

enum Item {
    TypeDecl(TaggedTypeDecl),
    Function(crate::ir::ast::FunDecl<()>),
}
