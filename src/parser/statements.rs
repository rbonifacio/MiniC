//! Statement parsers for MiniC.

use crate::ir::ast::Stmt;
use crate::parser::expressions::{expression, parse_call};
use crate::parser::identifiers::identifier;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

/// Parse any statement: if | while | call | block | assignment.
pub fn statement(input: &str) -> IResult<&str, Stmt> {
    preceded(
        multispace0,
        alt((
            if_statement,
            while_statement,
            call_statement,
            block_statement,
            assignment,
        )),
    )(input)
}

/// Parse a block statement: `{ stmt ; stmt ; ... }`.
fn block_statement(input: &str) -> IResult<&str, Stmt> {
    map(
        delimited(
            preceded(multispace0, char('{')),
            separated_list0(
                preceded(multispace0, char(';')),
                preceded(multispace0, statement),
            ),
            preceded(multispace0, char('}')),
        ),
        |seq| Stmt::Block { seq },
    )(input)
}

/// Parse a function call as a statement: `identifier ( expr_list )`.
fn call_statement(input: &str) -> IResult<&str, Stmt> {
    map(parse_call, |(name, args)| Stmt::Call { name, args })(input)
}

/// Parse an if-then-else statement: `if expr then stmt [else stmt]`.
fn if_statement(input: &str) -> IResult<&str, Stmt> {
    let (rest, _) = preceded(multispace0, tag("if"))(input)?;
    let (rest, cond) = preceded(multispace0, expression)(rest)?;
    let (rest, _) = preceded(multispace0, tag("then"))(rest)?;
    let (rest, then_branch) = preceded(multispace0, statement)(rest)?;
    let (rest, else_branch) = opt(map(
        tuple((
            preceded(multispace0, tag("else")),
            preceded(multispace0, statement),
        )),
        |(_, stmt)| stmt,
    ))(rest)?;
    Ok((
        rest,
        Stmt::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        },
    ))
}

/// Parse a while statement: `while expr do stmt`.
fn while_statement(input: &str) -> IResult<&str, Stmt> {
    let (rest, _) = preceded(multispace0, tag("while"))(input)?;
    let (rest, cond) = preceded(multispace0, expression)(rest)?;
    let (rest, _) = preceded(multispace0, tag("do"))(rest)?;
    let (rest, body) = preceded(multispace0, statement)(rest)?;
    Ok((
        rest,
        Stmt::While {
            cond: Box::new(cond),
            body: Box::new(body),
        },
    ))
}

/// Parse an assignment statement: `identifier = expression`.
pub fn assignment(input: &str) -> IResult<&str, Stmt> {
    map(
        tuple((
            preceded(multispace0, identifier),
            preceded(multispace0, nom::bytes::complete::tag("=")),
            preceded(multispace0, expression),
        )),
        |(target, _, value)| Stmt::Assign {
            target: target.to_string(),
            value: Box::new(value),
        },
    )(input)
}
