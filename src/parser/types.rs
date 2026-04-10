//! Shared type parsers for MiniC.
//!
//! This module defines the language's type syntax: base types, arrays, and
//! struct type names. It is reused by function parsing, struct field parsing,
//! and variable declarations.

use crate::ir::ast::{Member, TagType, TaggedTypeDecl, Type};
use crate::parser::identifiers::{identifier, identifier_decl};
use crate::parser::literals::integer_literal;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

fn member_field(input: &str) -> IResult<&str, Member> {
    map(
        tuple((
            preceded(multispace0, identifier_decl),
            preceded(multispace0, char(';')),
        )),
        |(decl, _)| Member::Field(decl),
    )(input)
}

fn enum_variant(input: &str) -> IResult<&str, Member> {
    map(
        tuple((
            preceded(multispace0, identifier),
            opt(preceded(
                preceded(multispace0, char('=')),
                preceded(multispace0, integer_literal),
            )),
            preceded(multispace0, char(';')),
        )),
        |(name, value, _)| Member::Enumerator {
            name: name.to_string(),
            value,
        },
    )(input)
}

fn tagged_type_and_name(input: &str) -> IResult<&str, (TagType, String)> {
    alt((
        map(
            tuple((
                preceded(multispace0, tag("struct")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (TagType::Struct, name.to_string()),
        ),
        map(
            tuple((
                preceded(multispace0, tag("union")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (TagType::Union, name.to_string()),
        ),
        map(
            tuple((
                preceded(multispace0, tag("enum")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (TagType::Enum, name.to_string()),
        ),
    ))(input)
}

/// Parse a tagged type: `[ struct | union | enum ] N {...}`.
pub fn tagged_type_decl(input: &str) -> IResult<&str, TaggedTypeDecl> {
    let (rest, (tag_type, tag_name)) = tagged_type_and_name(input)?;

    let (rest, members) = match tag_type {
        TagType::Struct | TagType::Union => delimited(
            preceded(multispace0, char('{')),
            many1(member_field),
            preceded(multispace0, char('}')),
        )(rest)?,
        TagType::Enum => delimited(
            preceded(multispace0, char('{')),
            many1(enum_variant),
            preceded(multispace0, char('}')),
        )(rest)?,
    };

    Ok((
        rest,
        TaggedTypeDecl {
            tag_type,
            tag_name,
            members,
        },
    ))
}

fn base_type(input: &str) -> IResult<&str, Type> {
    preceded(
        multispace0,
        alt((
            map(tagged_type_and_name, |(tag_type, tag_name)| Type::Tagged {
                tag_type,
                tag_name,
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
