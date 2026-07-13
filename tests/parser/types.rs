use mini_c::ir::ast::{IdentifierDecl, Statement, Type, UDTKind, UDTMember};
use mini_c::parser::identifiers::identifier_decl;
use mini_c::parser::statement;
use mini_c::parser::types::{type_definition, user_defined_type_decl};
use nom::combinator::all_consuming;

#[test]
fn user_defined_type_definition() {
    assert_eq!(
        type_definition("struct Point"),
        Ok(("", Type::Struct("Point".to_string())))
    );
    assert_eq!(
        type_definition("enum Kind"),
        Ok(("", Type::Enum("Kind".to_string())))
    );
    assert!(type_definition("union Value").is_err());
}

#[test]
fn user_defined_type_definition_array() {
    assert_eq!(
        type_definition("struct S[]"),
        Ok(("", Type::Array(Box::new(Type::Struct("S".to_string())))))
    );
}

#[test]
fn user_defined_type_identifier_decl() {
    assert_eq!(
        identifier_decl("struct Point p"),
        Ok((
            "",
            IdentifierDecl {
                name: "p".to_string(),
                ty: Type::Struct("Point".to_string())
            }
        ))
    );
    assert!(identifier_decl("union Value v").is_err());
    assert_eq!(
        identifier_decl("enum Kind k"),
        Ok((
            "",
            IdentifierDecl {
                name: "k".to_string(),
                ty: Type::Enum("Kind".to_string())
            }
        ))
    );
}

#[test]
fn struct_decl() {
    let result = all_consuming(user_defined_type_decl)("struct Point { int x; float y; }")
        .unwrap()
        .1;
    assert_eq!(result.specifier, UDTKind::Struct);
    assert_eq!(result.identifier, "Point");
    assert_eq!(result.members.len(), 2);
    assert_eq!(
        result.members[0],
        UDTMember::Field(IdentifierDecl {
            name: "x".into(),
            ty: Type::Int
        })
    );
    assert_eq!(
        result.members[1],
        UDTMember::Field(IdentifierDecl {
            name: "y".into(),
            ty: Type::Float
        })
    );
}

#[test]
fn union_decl_rejected() {
    assert!(all_consuming(user_defined_type_decl)("union Value { int i; float f; }").is_err());
}

#[test]
fn enum_decl() {
    let result = all_consuming(user_defined_type_decl)("enum Kind { OK; Err; }")
        .unwrap()
        .1;
    assert_eq!(result.specifier, UDTKind::Enum);
    assert_eq!(result.identifier, "Kind");
    assert_eq!(result.members.len(), 2);
    assert_eq!(
        result.members[0],
        UDTMember::EnumVariant {
            name: "OK".into(),
            ty: None
        }
    );
    assert_eq!(
        result.members[1],
        UDTMember::EnumVariant {
            name: "Err".into(),
            ty: None
        }
    );
}

#[test]
fn enum_with_payload_decl() {
    let result = all_consuming(user_defined_type_decl)("enum Option { int Some; None; }")
        .unwrap()
        .1;
    assert_eq!(result.members.len(), 2);
    assert_eq!(
        result.members[0],
        UDTMember::EnumVariant {
            name: "Some".into(),
            ty: Some(Type::Int)
        }
    );
    assert_eq!(
        result.members[1],
        UDTMember::EnumVariant {
            name: "None".into(),
            ty: None
        }
    );
}

#[test]
fn enum_with_payload_and_unit_variants() {
    let result = all_consuming(user_defined_type_decl)("enum E { float A; B; int C; }")
        .unwrap()
        .1;
    assert_eq!(result.members.len(), 3);
    assert_eq!(
        result.members[0],
        UDTMember::EnumVariant {
            name: "A".into(),
            ty: Some(Type::Float)
        }
    );
    assert_eq!(
        result.members[1],
        UDTMember::EnumVariant {
            name: "B".into(),
            ty: None
        }
    );
    assert_eq!(
        result.members[2],
        UDTMember::EnumVariant {
            name: "C".into(),
            ty: Some(Type::Int)
        }
    );
}

#[test]
fn str_type_decl() {
    let result = statement("str s = \"hello\";").unwrap().1;
    assert!(
        matches!(result.stmt, Statement::Decl { ref name, ref ty, .. } if name == "s" && ty == &Type::Str)
    );
}

#[test]
fn reject_empty_members() {
    assert!(all_consuming(user_defined_type_decl)("struct S { }").is_err());
    assert!(all_consuming(user_defined_type_decl)("enum E { }").is_err());
}

#[test]
fn reject_missing_member_semicolon() {
    assert!(all_consuming(user_defined_type_decl)("struct S { int x }").is_err());
    assert!(all_consuming(user_defined_type_decl)("enum E { int A }").is_err());
}

#[test]
fn reject_reserved_identifier_name() {
    assert!(all_consuming(user_defined_type_decl)("struct return { int x; }").is_err());
    assert!(all_consuming(user_defined_type_decl)("enum return { A; }").is_err());
}
