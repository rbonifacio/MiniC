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
//! Local `const T name = expr ;` is recognised inside function bodies via
//! [`crate::parser::statements::const_statement`] (wired from [`crate::parser::statements::statement`]).
//! Extending this module with `alt((const_statement, fun_decl))` plus a rich enough
//! [`crate::ir::ast::Program`] is the usual next step for top-level constants.
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

// use crate::ir::ast::{Program, UncheckedProgram};
// use crate::parser::functions::fun_decl;
// use nom::{combinator::map, multi::many0, IResult};
//
// /// Parse a complete MiniC program: zero or more function declarations.
// /// Execution starts at the `main` function (validated by the type checker).
// pub fn program(input: &str) -> IResult<&str, UncheckedProgram> {
//     map(many0(fun_decl), |functions| Program { 
//         functions, 
//         constants: vec![] 
//     })(input)
// }
// use crate::ir::ast::{Program, UncheckedProgram, FunDecl, StatementD};
// use crate::parser::functions::fun_decl;
// // Import the constant statement parser mentioned in your docs
// use crate::parser::statements::const_statement;
//
// use nom::{
//     branch::alt,
//     combinator::map,
//     multi::many0,
//     IResult,
// };
//
// /// An internal enum used to differentiate top-level elements during parsing.
// enum GlobalItem {
//     Function(FunDecl<()>),
//     Constant(StatementD<()>),
// }
//
// /// Parse a complete MiniC program: zero or more global constant declarations 
// /// and function declarations in any order.
// pub fn program(input: &str) -> IResult<&str, UncheckedProgram> {
//     // 1. Repeatedly parse either a function declaration OR a global constant
//     map(
//         many0(alt((
//             map(fun_decl, GlobalItem::Function),
//             map(const_statement, GlobalItem::Constant),
//         ))),
//         |items| {
//             let mut functions = Vec::new();
//             let mut constants = Vec::new();
//
//             // 2. Separate them into their respective vectors for the Program AST
//             for item in items {
//                 match item {
//                     GlobalItem::Function(f) => functions.push(f),
//                     GlobalItem::Constant(c) => constants.push(c),
//                 }
//             }
//
//             Program { functions, constants }
//         },
//     )(input)
// }
use crate::ir::ast::{Program, UncheckedProgram, FunDecl, StatementD};
use crate::parser::functions::fun_decl;
use crate::parser::statements::const_statement;

use nom::{
    branch::alt,
    combinator::map,
    multi::many0,
    character::complete::multispace0,
    sequence::delimited,
    IResult,
};

enum GlobalItem {
    Function(FunDecl<()>),
    Constant(StatementD<()>),
}

pub fn program(input: &str) -> IResult<&str, UncheckedProgram> {
    // Wrap the top level elements so that trailing or leading whitespace/newlines 
    // do not prematurely break `many0`
    map(
        many0(delimited(
            multispace0,
            alt((
                map(fun_decl, GlobalItem::Function),
                map(const_statement, GlobalItem::Constant),
            )),
            multispace0,
        )),
        |items| {
            let mut functions = Vec::new();
            let mut constants = Vec::new();

            for item in items {
                match item {
                    GlobalItem::Function(f) => functions.push(f),
                    GlobalItem::Constant(c) => constants.push(c),
                }
            }

            Program { functions, constants }
        },
    )(input)
}
