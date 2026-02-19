//! Program parser for MiniC.

use crate::ir::ast::Program;
use crate::parser::functions::fun_decl;
use crate::parser::statements::statement;
use nom::{combinator::map, multi::many0, sequence::tuple, IResult};

/// Parse a complete MiniC program: zero or more function declarations, then zero or more statements.
pub fn program(input: &str) -> IResult<&str, Program> {
    map(
        tuple((many0(fun_decl), many0(statement))),
        |(functions, body)| Program { functions, body },
    )(input)
}
