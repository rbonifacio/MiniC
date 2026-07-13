use mini_c::parser::literal;
use mini_c::parser::literals::{
    boolean_literal, float_literal, integer_literal, string_literal, Literal as ParserLiteral,
};

#[test]
fn integer_positive() {
    assert_eq!(integer_literal("42"), Ok(("", 42)));
    assert_eq!(integer_literal("0"), Ok(("", 0)));
    assert_eq!(integer_literal("12345"), Ok(("", 12345)));
}

#[test]
fn integer_negative() {
    assert_eq!(integer_literal("-17"), Ok(("", -17)));
    assert_eq!(integer_literal("-0"), Ok(("", 0)));
}

#[test]
fn integer_reject() {
    assert!(integer_literal("abc").is_err());
    assert!(integer_literal("12.34").is_err());
}

#[test]
fn float() {
    assert_eq!(float_literal("3.14"), Ok(("", 3.14)));
    assert_eq!(float_literal("0.5"), Ok(("", 0.5)));
    assert_eq!(float_literal("-0.25"), Ok(("", -0.25)));
}

#[test]
fn string_simple() {
    assert_eq!(string_literal(r#""hello""#), Ok(("", "hello".to_string())));
    assert_eq!(string_literal(r#""""#), Ok(("", "".to_string())));
}

#[test]
fn string_escapes() {
    assert_eq!(string_literal(r#""a\"b""#), Ok(("", "a\"b".to_string())));
    assert_eq!(
        string_literal(r#""line1\nline2""#),
        Ok(("", "line1\nline2".to_string()))
    );
    assert_eq!(
        string_literal(r#""tab\there""#),
        Ok(("", "tab\there".to_string()))
    );
}

#[test]
fn boolean() {
    assert_eq!(boolean_literal("true"), Ok(("", true)));
    assert_eq!(boolean_literal("false"), Ok(("", false)));
}

#[test]
fn boolean_reject() {
    assert!(boolean_literal("True").is_err());
    assert!(boolean_literal("1").is_err());
}

#[test]
fn literal_combined() {
    assert_eq!(literal("42"), Ok(("", ParserLiteral::Int(42))));
    assert_eq!(literal("3.14"), Ok(("", ParserLiteral::Float(3.14))));
    assert_eq!(
        literal(r#""hi""#),
        Ok(("", ParserLiteral::Str("hi".to_string())))
    );
    assert_eq!(literal("true"), Ok(("", ParserLiteral::Bool(true))));
}
