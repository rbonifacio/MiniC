//! Expression parsers for MiniC.

use crate::ir::ast::Expr;
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

/// Parse a function call: `identifier ( expr_list )`. Returns (name, args).
pub fn parse_call(input: &str) -> IResult<&str, (String, Vec<Expr>)> {
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

/// Primary: literal, call, identifier, or parenthesized expression.
fn primary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(literal, |l| Expr::Literal(l.into())),
        map(parse_call, |(name, args)| Expr::Call { name, args }),
        map(identifier, |s: &str| Expr::Ident(s.to_string())),
        delimited(
            preceded(multispace0, char('(')),
            preceded(multispace0, expression),
            preceded(multispace0, char(')')),
        ),
    ))(input)
}

/// Unary: optional unary `-` applied to primary.
fn unary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            pair(preceded(multispace0, tag("-")), unary),
            |(_, e)| Expr::Neg(Box::new(e)),
        ),
        primary,
    ))(input)
}

/// Multiplicative: unary with `*` and `/` (left-associative).
fn multiplicative(input: &str) -> IResult<&str, Expr> {
    let (mut rest, mut acc) = unary(input)?;
    loop {
        let mul = tuple((multispace0, tag("*"), multispace0, unary))(rest);
        if let Ok((r, (_, _, _, e))) = mul {
            acc = Expr::Mul(Box::new(acc), Box::new(e));
            rest = r;
            continue;
        }
        let div = tuple((multispace0, tag("/"), multispace0, unary))(rest);
        if let Ok((r, (_, _, _, e))) = div {
            acc = Expr::Div(Box::new(acc), Box::new(e));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Additive: multiplicative with `+` and `-` (left-associative).
fn additive(input: &str) -> IResult<&str, Expr> {
    let (mut rest, mut acc) = multiplicative(input)?;
    loop {
        let add = tuple((multispace0, tag("+"), multispace0, multiplicative))(rest);
        if let Ok((r, (_, _, _, e))) = add {
            acc = Expr::Add(Box::new(acc), Box::new(e));
            rest = r;
            continue;
        }
        let sub = tuple((multispace0, tag("-"), multispace0, multiplicative))(rest);
        if let Ok((r, (_, _, _, e))) = sub {
            acc = Expr::Sub(Box::new(acc), Box::new(e));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Relational: additive with ==, !=, <, <=, >, >=
fn relational(input: &str) -> IResult<&str, Expr> {
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
                acc = match op {
                    "==" => Expr::Eq(Box::new(acc), Box::new(e)),
                    "!=" => Expr::Ne(Box::new(acc), Box::new(e)),
                    "<" => Expr::Lt(Box::new(acc), Box::new(e)),
                    "<=" => Expr::Le(Box::new(acc), Box::new(e)),
                    ">" => Expr::Gt(Box::new(acc), Box::new(e)),
                    ">=" => Expr::Ge(Box::new(acc), Box::new(e)),
                    _ => unreachable!(),
                };
                rest = r;
            }
            Err(_) => break,
        }
    }
    Ok((rest, acc))
}

/// Logical not: optional `!` applied to relational.
fn logical_not(input: &str) -> IResult<&str, Expr> {
    alt((
        map(
            pair(
                preceded(multispace0, char('!')),
                preceded(multispace0, logical_not),
            ),
            |(_, e)| Expr::Not(Box::new(e)),
        ),
        relational,
    ))(input)
}

/// Logical and: logical_not with `and` (left-associative).
fn logical_and(input: &str) -> IResult<&str, Expr> {
    let (mut rest, mut acc) = logical_not(input)?;
    loop {
        let and = tuple((multispace0, tag("and"), multispace0, logical_not))(rest);
        if let Ok((r, (_, _, _, e))) = and {
            acc = Expr::And(Box::new(acc), Box::new(e));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Logical or: logical_and with `or` (left-associative).
fn logical_or(input: &str) -> IResult<&str, Expr> {
    let (mut rest, mut acc) = logical_and(input)?;
    loop {
        let or = tuple((multispace0, tag("or"), multispace0, logical_and))(rest);
        if let Ok((r, (_, _, _, e))) = or {
            acc = Expr::Or(Box::new(acc), Box::new(e));
            rest = r;
            continue;
        }
        break;
    }
    Ok((rest, acc))
}

/// Top-level expression parser.
pub fn expression(input: &str) -> IResult<&str, Expr> {
    preceded(multispace0, logical_or)(input)
}
