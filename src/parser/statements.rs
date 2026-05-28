//! Statement parsers for MiniC.
//!
//! # Overview
//!
//! Exposes two public functions:
//!
//! * [`statement`] — the top-level entry point; tries each statement form in
//!   order: `return`, `if`, `while`, call-statement, block, declaration,
//!   assignment.
//! * [`assignment`] — parses `lvalue = expression ;`; exported separately
//!   because the test suite uses it directly.
//!
//! # Grammar
//!
//! ```text
//! statement  := block | if_stmt | while_stmt | simple ';'
//! block      := '{' statement* '}'
//! if_stmt    := 'if' expr block ['else' block]
//! while_stmt := 'while' expr block
//! simple     := return | decl | call | assign
//! ```
//!
//! Every simple statement is terminated by `;`.
//! Compound statements (`if`, `while`, block) end with `}` and need no `;`.
//!
//! # Design Decisions
//!
//! ## Declaration must be tried before assignment
//!
//! Both `int x = 0` (declaration) and `x = 0` (assignment) begin with an
//! identifier-like token, so the order of alternatives in [`statement`]
//! matters. Declaration is tried first because it starts with a type keyword
//! (`int`, `float`, …), which is unambiguous. If declaration fails, the
//! parser backtracks and tries assignment.
//!
//! ## `lvalue` handles nested array indexing on the left-hand side
//!
//! An assignment target can be a plain variable (`x = …`) or a nested array
//! element (`a[i][j] = …`). The private `lvalue` parser accumulates index
//! suffixes in a loop using the same pattern as the `primary` parser in
//! `expressions.rs`, producing a left-associative `Index` chain.

use crate::ir::ast::{Expr, ExprD, Statement, StatementD, UncheckedExpr, UncheckedStmt};
use crate::parser::expressions::{expression, parse_call};
use crate::parser::functions::type_name;
use crate::parser::identifiers::identifier;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

fn wrap(s: Statement<()>) -> UncheckedStmt {
    StatementD { stmt: s, ty: () }
}

/// Parse any statement: block | if | while | return | decl | call | assignment.
pub fn statement(input: &str) -> IResult<&str, UncheckedStmt> {
    preceded(
        multispace0,
        alt((
            block_statement,
            if_statement,
            while_statement,
            return_statement,
            decl_statement,
            call_statement,
            assignment,
        )),
    )(input)
}

/// Parse a return statement: `return [expr] ;`.
fn return_statement(input: &str) -> IResult<&str, UncheckedStmt> {
    let (rest, _) = preceded(multispace0, tag("return"))(input)?;
    let (rest, expr) = opt(preceded(multispace0, expression))(rest)?;
    let (rest, _) = preceded(multispace0, char(';'))(rest)?;
    Ok((rest, wrap(Statement::Return(expr.map(Box::new)))))
}

/// Parse a variable declaration: `Type ident = expr ;`. Must come before assignment.
fn decl_statement(input: &str) -> IResult<&str, UncheckedStmt> {
    map(
        tuple((
            type_name,
            preceded(nom::character::complete::multispace1, identifier),
            preceded(multispace0, nom::bytes::complete::tag("=")),
            preceded(multispace0, expression),
            preceded(multispace0, char(';')),
        )),
        |(ty, name, _, init, _)| {
            wrap(Statement::Decl {
                name: name.to_string(),
                ty,
                init: Box::new(init),
            })
        },
    )(input)
}

/// Parse a block statement: `{ stmt* }`.
/// Each statement inside the block carries its own terminator (`;` or `}`).
fn block_statement(input: &str) -> IResult<&str, UncheckedStmt> {
    map(
        delimited(
            preceded(multispace0, char('{')),
            many0(preceded(multispace0, statement)),
            preceded(multispace0, char('}')),
        ),
        |seq| wrap(Statement::Block { seq }),
    )(input)
}

/// Parse a function call as a statement: `identifier ( expr_list ) ;`.
fn call_statement(input: &str) -> IResult<&str, UncheckedStmt> {
    let (rest, (name, args)) = parse_call(input)?;
    let (rest, _) = preceded(multispace0, char(';'))(rest)?;
    Ok((rest, wrap(Statement::Call { name, args })))
}

/// Parse an if statement: `if expr block ['else' block]`.
/// Both branches must be blocks — bare statements are not allowed.
fn if_statement(input: &str) -> IResult<&str, UncheckedStmt> {
    let (rest, _) = preceded(multispace0, tag("if"))(input)?;
    let (rest, cond) = preceded(multispace0, expression)(rest)?;
    let (rest, then_branch) = preceded(multispace0, block_statement)(rest)?;
    let (rest, else_branch) = opt(map(
        tuple((
            preceded(multispace0, tag("else")),
            preceded(multispace0, block_statement),
        )),
        |(_, stmt)| stmt,
    ))(rest)?;
    Ok((
        rest,
        wrap(Statement::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }),
    ))
}

/// Parse a while statement: `while expr block`.
/// The body must be a block — bare statements are not allowed.
fn while_statement(input: &str) -> IResult<&str, UncheckedStmt> {
    let (rest, _) = preceded(multispace0, tag("while"))(input)?;
    let (rest, cond) = preceded(multispace0, expression)(rest)?;
    let (rest, body) = preceded(multispace0, block_statement)(rest)?;
    Ok((
        rest,
        wrap(Statement::While {
            cond: Box::new(cond),
            body: Box::new(body),
        }),
    ))
}

/// Parse an lvalue: identifier followed by zero or more `[ expr ]` suffixes.
fn lvalue(input: &str) -> IResult<&str, UncheckedExpr> {
    let (mut rest, mut acc) = preceded(
        multispace0,
        alt((
            map(preceded(tag("*"), identifier), |id| ExprD {
                exp: Expr::Deref(Box::new(ExprD {
                    exp: Expr::Ident(id.to_string()),
                    ty: (),
                })),
                ty: (),
            }),
            map(identifier, |id| ExprD {
                exp: Expr::Ident(id.to_string()),
                ty: (),
            }),
        )),
    )(input)?;
    loop {
        let index_parse = delimited(
            preceded(multispace0, char('[')),
            preceded(multispace0, expression),
            preceded(multispace0, char(']')),
        )(rest);
        match index_parse {
            Ok((r, index)) => {
                acc = ExprD {
                    exp: Expr::Index {
                        base: Box::new(acc),
                        index: Box::new(index),
                    },
                    ty: (),
                };
                rest = r;
            }
            Err(_) => break,
        }
    }
    Ok((rest, acc))
}

/// Parse an assignment statement: `lvalue = expression ;`.
pub fn assignment(input: &str) -> IResult<&str, UncheckedStmt> {
    map(
        tuple((
            lvalue,
            preceded(multispace0, nom::bytes::complete::tag("=")),
            preceded(multispace0, expression),
            preceded(multispace0, char(';')),
        )),
        |(target, _, value, _)| {
            wrap(Statement::Assign {
                target: Box::new(target),
                value: Box::new(value),
            })
        },
    )(input)
}