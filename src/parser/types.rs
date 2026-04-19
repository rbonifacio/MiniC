//! Shared type parsers for MiniC.
//!
//! This module defines the language's type syntax: base types, arrays, and
//! struct type names. It is reused by function parsing, struct field parsing,
//! and variable declarations.

use crate::ir::ast::{AggregateTypeDecl, AgtTypeMember, AgtTypeSpecifier, Type};
use crate::parser::identifiers::{identifier, identifier_decl};
use crate::parser::literals::integer_literal;
use nom::multi::many1;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

fn agt_member_field(input: &str) -> IResult<&str, AgtTypeMember> {
    map(
        tuple((
            preceded(multispace0, identifier_decl),
            preceded(multispace0, char(';')),
        )),
        |(decl, _)| AgtTypeMember::Field(decl),
    )(input)
}

fn agt_member_enumerator(input: &str) -> IResult<&str, AgtTypeMember> {
    map(
        tuple((
            preceded(multispace0, identifier),
            opt(preceded(
                preceded(multispace0, char('=')),
                preceded(multispace0, integer_literal),
            )),
            preceded(multispace0, char(';')),
        )),
        |(name, value, _)| AgtTypeMember::Enumerator {
            name: name.to_string(),
            value,
        },
    )(input)
}

fn aggregate_type_name(input: &str) -> IResult<&str, (AgtTypeSpecifier, String)> {
    alt((
        map(
            tuple((
                preceded(multispace0, tag("struct")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (AgtTypeSpecifier::Struct, name.to_string()),
        ),
        map(
            tuple((
                preceded(multispace0, tag("union")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (AgtTypeSpecifier::Union, name.to_string()),
        ),
        map(
            tuple((
                preceded(multispace0, tag("enum")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (AgtTypeSpecifier::Enum, name.to_string()),
        ),
    ))(input)
}

/// Parse an aggregate type: `[ struct | union | enum ] N {...}`.
pub fn aggregate_type_decl(input: &str) -> IResult<&str, AggregateTypeDecl> {
    let (rest, (specifier, identifier)) = aggregate_type_name(input)?;

    let member_parser = match specifier {
        AgtTypeSpecifier::Struct | AgtTypeSpecifier::Union => agt_member_field,
        AgtTypeSpecifier::Enum => agt_member_enumerator,
    };

    let (rest, members) = delimited(
        preceded(multispace0, char('{')),
        many1(preceded(multispace0, member_parser)),
        preceded(multispace0, char('}')),
    )(rest)?;

    Ok((
        rest,
        AggregateTypeDecl {
            specifier,
            identifier,
            members,
        },
    ))
}

fn base_type(input: &str) -> IResult<&str, Type> {
    preceded(
        multispace0,
        alt((
            map(aggregate_type_name, |(specifier, identifier)| {
                Type::Aggregate {
                    specifier,
                    identifier,
                }
            }),
            map(tag("int"), |_| Type::Int),
            map(tag("float"), |_| Type::Float),
            map(tag("bool"), |_| Type::Bool),
            map(tag("str"), |_| Type::Str),
            map(tag("void"), |_| Type::Unit),
        )),
    )(input)
}

/// Parse a type name: int | float | bool | str | void | struct N | union N | enum N | T[] | T[][].
pub fn type_definition(input: &str) -> IResult<&str, Type> {
    map(pair(base_type, many0(tag("[]"))), |(base, dimensions)| {
        dimensions
            .into_iter()
            .fold(base, |inner, _| Type::Array(Box::new(inner)))
    })(input)
}
