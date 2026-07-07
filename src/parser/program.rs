//! Top-level program parser for MiniC.
//!
//! # Overview
//!
//! Exposes one public function:
//!
//! * [`program`] — parses a complete MiniC program as a sequence of zero or
//!   more function declarations and test blocks, returning an
//!   [`UncheckedProgram`].
//!
//! A valid MiniC program contains only function declarations and `test` blocks
//! at the top level. The type checker verifies that a `main` function exists
//! (required in `--run` mode).
//!
//! # Design Decisions
//!
//! ## `many0` as the top-level combinator
//!
//! `nom`'s `many0` combinator repeatedly applies a parser until it fails,
//! collecting results in a `Vec`. Using it here means the program parser
//! naturally handles empty programs and programs with any number of
//! top-level items with no extra branching logic.

use crate::ir::ast::{Program, TestDecl, UncheckedProgram, UncheckedTestDecl};
use crate::parser::functions::fun_decl;
use crate::parser::statements::statement;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::map,
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};

enum TopItem {
    Fun(crate::ir::ast::UncheckedFunDecl),
    Test(UncheckedTestDecl),
}

fn test_decl(input: &str) -> IResult<&str, UncheckedTestDecl> {
    let (rest, _) = preceded(multispace0, tag("test"))(input)?;
    let (rest, name) = preceded(
        multispace0,
        delimited(char('"'), nom::bytes::complete::take_while(|c| c != '"'), char('"')),
    )(rest)?;
    let (rest, body) = preceded(
        multispace0,
        map(
            delimited(
                preceded(multispace0, char('{')),
                many0(statement),
                preceded(multispace0, char('}')),
            ),
            |seq| crate::ir::ast::StatementD {
                stmt: crate::ir::ast::Statement::Block { seq },
                ty: (),
            },
        ),
    )(rest)?;
    Ok((
        rest,
        TestDecl {
            name: name.to_string(),
            body: Box::new(body),
        },
    ))
}

fn top_item(input: &str) -> IResult<&str, TopItem> {
    preceded(
        multispace0,
        alt((
            map(test_decl, TopItem::Test),
            map(fun_decl, TopItem::Fun),
        )),
    )(input)
}

/// Parse a complete MiniC program: zero or more function declarations and test blocks.
pub fn program(input: &str) -> IResult<&str, UncheckedProgram> {
    map(many0(top_item), |items| {
        let mut functions = Vec::new();
        let mut tests = Vec::new();
        for item in items {
            match item {
                TopItem::Fun(f) => functions.push(f),
                TopItem::Test(t) => tests.push(t),
            }
        }
        Program { functions, tests }
    })(input)
}
