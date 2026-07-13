//! Shared type parsers for MiniC.
//!
//! This module defines the language's type syntax: base types, arrays, and
//! struct type names. It is reused by function parsing, struct field parsing,
//! and variable declarations.

use crate::ir::ast::{Type, UDTDecl, UDTKind, UDTMember};
use crate::parser::identifiers::{identifier, identifier_decl};
use nom::multi::many1;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, multispace1},
    combinator::map,
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

fn member_field(input: &str) -> IResult<&str, UDTMember> {
    map(
        tuple((
            preceded(multispace0, identifier_decl),
            preceded(multispace0, char(';')),
        )),
        |(decl, _)| UDTMember::Field(decl),
    )(input)
}

fn member_enum_variant(input: &str) -> IResult<&str, UDTMember> {
    alt((
        // Payload variant: `type name ;`
        map(
            tuple((
                preceded(multispace0, type_definition),
                preceded(multispace1, identifier),
                preceded(multispace0, char(';')),
            )),
            |(ty, name, _)| UDTMember::EnumVariant {
                name: name.to_string(),
                ty: Some(ty),
            },
        ),
        // Unit variant: `name ;`
        map(
            tuple((
                preceded(multispace0, identifier),
                preceded(multispace0, char(';')),
            )),
            |(name, _)| UDTMember::EnumVariant {
                name: name.to_string(),
                ty: None,
            },
        ),
    ))(input)
}

fn user_defined_type_name(input: &str) -> IResult<&str, (UDTKind, String)> {
    alt((
        map(
            tuple((
                preceded(multispace0, tag("struct")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (UDTKind::Struct, name.to_string()),
        ),
        map(
            tuple((
                preceded(multispace0, tag("enum")),
                preceded(multispace1, identifier),
            )),
            |(_, name)| (UDTKind::Enum, name.to_string()),
        ),
    ))(input)
}

/// Parse a user-defined type declaration: `[ struct | enum ] N {...}`.
pub fn user_defined_type_decl(input: &str) -> IResult<&str, UDTDecl> {
    let (rest, (specifier, identifier)) = user_defined_type_name(input)?;

    let member_parser = match specifier {
        UDTKind::Struct => member_field,
        UDTKind::Enum => member_enum_variant,
    };

    let (rest, members) = delimited(
        preceded(multispace0, char('{')),
        many1(preceded(multispace0, member_parser)),
        preceded(multispace0, char('}')),
    )(rest)?;

    Ok((
        rest,
        UDTDecl {
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
            map(
                pair(tag("struct"), preceded(multispace1, identifier)),
                |(_, name)| Type::Struct(name.to_string()),
            ),
            map(
                pair(tag("enum"), preceded(multispace1, identifier)),
                |(_, name)| Type::Enum(name.to_string()),
            ),
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
