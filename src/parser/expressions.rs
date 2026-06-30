//! Expression parsers for MiniC.
//!
//! # Overview
//!
//! Exposes two public functions:
//!
//! * [`expression`] — the top-level entry point; parses any MiniC expression.
//! * [`parse_call`] — parses a function call `name(arg, …)`; re-used by the
//!   statement parser to handle call-statements.
//!
//! # Design Decisions
//!
//! ## Precedence via a parser chain
//!
//! Operator precedence is encoded by the order in which parsers call each
//! other, from lowest to highest:
//!
//! ```text
//! expression → logical_or → logical_and → logical_not
//!           → relational → additive → multiplicative
//!           → unary → primary → atom
//! ```
//!
//! Each level calls the level above it for its operands, which naturally
//! gives higher-precedence operators tighter binding — the same technique
//! used in hand-written recursive-descent parsers. No separate precedence
//! table or Pratt parser is needed for the small MiniC operator set.
//!
//! ## Left-associativity via an accumulator loop
//!
//! Operators at the same precedence level (e.g., `+` and `-`) are
//! left-associative: `1 - 2 - 3` means `(1 - 2) - 3`. This is implemented
//! with an explicit `loop` that accumulates results into `acc` rather than
//! recursing on the right-hand side, which would accidentally produce
//! right-associative trees.

use crate::ir::ast::{Expr, ExprD, UncheckedExpr};
use crate::parser::identifiers::identifier;
use crate::parser::literals::literal;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0},
    combinator::map,
    multi::separated_list0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

fn wrap(e: Expr<()>) -> UncheckedExpr {
    ExprD { exp: e, ty: () }
}

/// Parse a function call: `identifier ( expr_list )`. Returns (name, args).
pub fn parse_call(input: &str) -> IResult<&str, (String, Vec<UncheckedExpr>)> {
    let (rest, name) = preceded(multispace0, identifier)(input)?;
    let (rest, args) = delimited(
        preceded(multispace0, tag("(")),
        separated_list0(
            preceded(multispace0, tag(",")),
            preceded(multispace0, expression),
        ),
        preceded(multispace0, tag(")")),
    )(rest)?;
    Ok((rest, (name.to_string(), args)))
}

/// Atom: literal, call, array literal, identifier, or parenthesized expression.
fn atom(input: &str) -> IResult<&str, UncheckedExpr> {
    alt((
        map(literal, |l| wrap(Expr::Literal(l.into()))),
        map(parse_call, |(name, args)| wrap(Expr::Call { name, args })),
        map(
            delimited(
                preceded(multispace0, char('[')),
                separated_list0(
                    preceded(multispace0, tag(",")),
                    preceded(multispace0, expression),
                ),
                preceded(multispace0, char(']')),
            ),
            |elems| wrap(Expr::ArrayLit(elems)),
        ),
        map(identifier, |s: &str| wrap(Expr::Ident(s.to_string()))),
        delimited(
            preceded(multispace0, char('(')),
            preceded(multispace0, expression),
            preceded(multispace0, char(')')),
        ),
    ))(input)
}

/// Primary: atom with zero or more index postfixes `[ expr ]`.
fn primary(input: &str) -> IResult<&str, UncheckedExpr> {
    let (mut rest, mut acc) = atom(input)?;
    loop {
        let index_parse = delimited(
            preceded(multispace0, char('[')),
            preceded(multispace0, expression),
            preceded(multispace0, char(']')),
        )(rest);
        match index_parse {
            Ok((r, index)) => {
                acc = wrap(Expr::Index {
                    base: Box::new(acc),
                    index: Box::new(index),
                });
                rest = r;
            }
            Err(_) => break,
        }
    }
    Ok((rest, acc))
}

/// Unary: optional unary `-` applied to primary.
fn unary(input: &str) -> IResult<&str, UncheckedExpr> {
    alt((
        map(pair(preceded(multispace0, tag("-")), unary), |(_, e)| {
            wrap(Expr::Neg(Box::new(e)))
        }),
        map(pair(preceded(multispace0, tag("&")), unary), |(_, e)| {
            wrap(Expr::AddrOf(Box::new(e)))
        }),
        map(pair(preceded(multispace0, tag("*")), unary), |(_, e)| {
            wrap(Expr::Deref(Box::new(e)))
        }),
        primary,
    ))(input)
}

/// Multiplicative: unary with `*` and `/` (left-associative).
fn multiplicative(input: &str) -> IResult<&str, UncheckedExpr> {
    let (mut rest, mut acc) = unary(input)?;
    loop {
        let mul = tuple((multispace0, tag("*"), multispace0, unary))(rest);
        if let Ok((r, (_, _, _, e))) = mul {
            acc = wrap(Expr::Mul(Box::new(acc), Box::new(e)));
            rest = r;
            continue;
        }
        let div = tuple((multispace0, tag("/"), multispace0, unary))(rest);
        if let Ok((r, (_, _, _, e))) = div {
            acc = wrap(Expr::Div(Box::new(acc), Box::new(e)));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Additive: multiplicative with `+` and `-` (left-associative).
fn additive(input: &str) -> IResult<&str, UncheckedExpr> {
    let (mut rest, mut acc) = multiplicative(input)?;
    loop {
        let add = tuple((multispace0, tag("+"), multispace0, multiplicative))(rest);
        if let Ok((r, (_, _, _, e))) = add {
            acc = wrap(Expr::Add(Box::new(acc), Box::new(e)));
            rest = r;
            continue;
        }
        let sub = tuple((multispace0, tag("-"), multispace0, multiplicative))(rest);
        if let Ok((r, (_, _, _, e))) = sub {
            acc = wrap(Expr::Sub(Box::new(acc), Box::new(e)));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Relational: additive with ==, !=, <, <=, >, >=
fn relational(input: &str) -> IResult<&str, UncheckedExpr> {
    let (mut rest, mut acc) = additive(input)?;
    loop {
        let ops = alt((
            tuple((multispace0, tag("=="), multispace0, additive)),
            tuple((multispace0, tag("!="), multispace0, additive)),
            tuple((multispace0, tag("<="), multispace0, additive)),
            tuple((multispace0, tag(">="), multispace0, additive)),
            tuple((multispace0, tag("<"), multispace0, additive)),
            tuple((multispace0, tag(">"), multispace0, additive)),
        ))(rest);
        match ops {
            Ok((r, (_, op, _, e))) => {
                acc = wrap(match op {
                    "==" => Expr::Eq(Box::new(acc), Box::new(e)),
                    "!=" => Expr::Ne(Box::new(acc), Box::new(e)),
                    "<" => Expr::Lt(Box::new(acc), Box::new(e)),
                    "<=" => Expr::Le(Box::new(acc), Box::new(e)),
                    ">" => Expr::Gt(Box::new(acc), Box::new(e)),
                    ">=" => Expr::Ge(Box::new(acc), Box::new(e)),
                    _ => unreachable!(),
                });
                rest = r;
            }
            Err(_) => break,
        }
    }
    Ok((rest, acc))
}

/// Logical not: optional `!` applied to relational.
fn logical_not(input: &str) -> IResult<&str, UncheckedExpr> {
    alt((
        map(
            pair(
                preceded(multispace0, char('!')),
                preceded(multispace0, logical_not),
            ),
            |(_, e)| wrap(Expr::Not(Box::new(e))),
        ),
        relational,
    ))(input)
}

/// Logical and: logical_not with `and` (left-associative).
fn logical_and(input: &str) -> IResult<&str, UncheckedExpr> {
    let (mut rest, mut acc) = logical_not(input)?;
    loop {
        let and = tuple((multispace0, tag("and"), multispace0, logical_not))(rest);
        if let Ok((r, (_, _, _, e))) = and {
            acc = wrap(Expr::And(Box::new(acc), Box::new(e)));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Logical or: logical_and with `or` (left-associative).
fn logical_or(input: &str) -> IResult<&str, UncheckedExpr> {
    let (mut rest, mut acc) = logical_and(input)?;
    loop {
        let or = tuple((multispace0, tag("or"), multispace0, logical_and))(rest);
        if let Ok((r, (_, _, _, e))) = or {
            acc = wrap(Expr::Or(Box::new(acc), Box::new(e)));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Top-level expression parser. Returns UncheckedExpr with ty: () at each node.
pub fn expression(input: &str) -> IResult<&str, UncheckedExpr> {
    preceded(multispace0, logical_or)(input)
}
