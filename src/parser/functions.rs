//! Function declaration parser for MiniC.
//!
//! # Overview
//!
//! Exposes one public function:
//!
//! * [`fun_decl`] — parses a complete function declaration in C style:
//!   `ReturnType name(Type param, …) body`. The body is any single
//!   statement (typically a block `{ … }`).
//!
//! # Design Decisions
//!
//! ## Type syntax is a shared, lower-level concept
//!
//! Function parsing should reuse the same `type_definition` parser as other
//! declaration forms. That keeps grammar ownership aligned with language
//! concepts rather than implementation convenience.

use crate::ir::ast::{FunDecl, UncheckedFunDecl};
use crate::parser::identifiers::{identifier, identifier_decl};
use crate::parser::statements::statement;
use crate::parser::types::type_definition;
use nom::{
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    multi::separated_list0,
    sequence::{delimited, preceded},
    IResult,
};

/// Parse a function declaration (C-style): `ReturnType name(Type name, ...) body`.
/// Example: `int add(int x, int y) { ... }` or `void main() x = 1`.
pub fn fun_decl(input: &str) -> IResult<&str, UncheckedFunDecl> {
    let (rest, return_type) = preceded(multispace0, type_definition)(input)?;
    let (rest, name) = preceded(multispace1, identifier)(rest)?;
    let (rest, params) = delimited(
        preceded(multispace0, tag("(")),
        separated_list0(preceded(multispace0, tag(",")), identifier_decl),
        preceded(multispace0, tag(")")),
    )(rest)?;
    let (rest, body) = preceded(multispace0, statement)(rest)?;
    Ok((
        rest,
        FunDecl {
            name: name.to_string(),
            params,
            return_type,
            body: Box::new(body),
        },
    ))
}
