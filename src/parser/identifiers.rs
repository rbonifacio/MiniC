//! Identifier parser for MiniC.
//!
//! # Overview
//!
//! Exposes one public function:
//!
//! * [`identifier`] — parses a valid MiniC identifier: starts with a letter
//!   or underscore, followed by letters, digits, or underscores. Rejects
//!   reserved words (`true`, `false`, `int`, `float`, `bool`, `str`, `void`,
//!   `return`).
//!
//! # Design Decisions
//!
//! ## Rejecting reserved words with `verify`
//!
//! `nom`'s `verify` combinator runs a predicate on the parsed result and
//! fails the parser if the predicate returns `false`. This is the cleanest
//! way to layer a keyword-exclusion rule on top of a general pattern match:
//! the character-class rule is written once, and the reserved-word check is
//! added as a separate, readable step. The alternative — listing every
//! keyword as a negative lookahead — would be more fragile to maintain.

use nom::{
    bytes::complete::{take_while, take_while1},
    combinator::{recognize, verify},
    sequence::pair,
    IResult,
};

/// Reserved words: boolean literals and type names.
const RESERVED: &[&str] = &[
    "true", "false", "int", "float", "bool", "str", "void", "return", "assert", "test",
];

/// Parse an identifier (variable name).
/// Must start with letter or underscore; subsequent chars may be letter, digit, or underscore.
/// Rejects reserved words (true, false, int, float, bool, str, void).
pub fn identifier(input: &str) -> IResult<&str, &str> {
    let id_parser = recognize(pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ));
    verify(id_parser, |s: &str| !RESERVED.contains(&s))(input)
}
