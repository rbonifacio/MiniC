//! Function declaration parser for MiniC.

use crate::ir::ast::FunDecl;
use crate::parser::identifiers::identifier;
use crate::parser::statements::statement;
use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    multi::separated_list0,
    sequence::{delimited, preceded},
    IResult,
};

/// Parse a function declaration: `def name(params) body`.
pub fn fun_decl(input: &str) -> IResult<&str, FunDecl> {
    let (rest, _) = preceded(multispace0, tag("def"))(input)?;
    let (rest, name) = preceded(multispace0, identifier)(rest)?;
    let (rest, params) = delimited(
        preceded(multispace0, tag("(")),
        separated_list0(
            preceded(multispace0, tag(",")),
            preceded(multispace0, identifier),
        ),
        preceded(multispace0, tag(")")),
    )(rest)?;
    let (rest, body) = preceded(multispace0, statement)(rest)?;
    Ok((
        rest,
        FunDecl {
            name: name.to_string(),
            params: params.iter().map(|s| s.to_string()).collect(),
            body: Box::new(body),
        },
    ))
}
