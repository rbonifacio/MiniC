use super::helpers::parse_and_type_check;

#[test]
fn struct_decl_and_member_access() {
    assert!(parse_and_type_check(
        "struct Point { int x; int y; }\nvoid main() { struct Point p = { .x = 12, .y = 0 }; int v = p.x; }",
    ).is_ok());
}

#[test]
fn enum_decl_and_init() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .Some = 42 }; }",
    )
    .is_ok());
}

#[test]
fn match_on_enum() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .Some = 42 }; match x { case Some: { int y = Some; } case None: { int z = 0; } } }",
    ).is_ok());
}

#[test]
fn struct_unknown_member() {
    let result = parse_and_type_check(
        "struct Point { int x; }\nvoid main() { struct Point p = { .x = 0 }; int v = p.y; }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown member"));
}

#[test]
fn enum_member_access() {
    let result = parse_and_type_check(
        "enum Color { Red; Green; }\nvoid main() { enum Color c = { .Red }; int v = c.Red; }",
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("cannot access enum variants directly"));
}

#[test]
fn unknown_type_declaration_use() {
    let result = parse_and_type_check("void main() { struct Missing x = { .x = 0 }; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown struct type"));
}

#[test]
fn member_access_on_non_struct() {
    let result = parse_and_type_check("void main() { int x = 0; int y = x.foo; }");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("member access requires struct base type"));
}

#[test]
fn duplicate_type_declarations() {
    let result = parse_and_type_check(
        "struct Point { int x; }\nstruct Point { int y; }\nvoid main() { int z = 0; }",
    );
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .message
        .contains("duplicate type declaration"));
}

#[test]
fn duplicate_struct_init_fields() {
    let result = parse_and_type_check(
        "struct Point { int x; int y; }\nvoid main() { struct Point p = { .x = 1, .x = 2, .y = 3 }; }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("duplicate field"));
}

#[test]
fn cast_mismatch_in_enum_decl() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = (int){ .Some = 42 }; }",
    )
    .is_err());
}

#[test]
fn unit_variant_with_payload() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .None = 42 }; }",
    )
    .is_err());
}

#[test]
fn payload_variant_without_arg() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .Some }; }",
    )
    .is_err());
}

#[test]
fn cast_in_expression() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid foo(enum Option x) { }\nvoid main() { foo((enum Option){ .Some = 42 }); }",
    ).is_ok());
}

#[test]
fn nested_struct_init() {
    assert!(parse_and_type_check(
        "struct Inner { int x; }\nstruct Outer { struct Inner inner; }\nvoid main() { struct Outer o = { .inner = { .x = 42 } }; }",
    ).is_ok());
}

#[test]
fn struct_init_in_call() {
    assert!(parse_and_type_check(
        "struct Point { int x; int y; }\nvoid foo(struct Point p) { }\nvoid main() { foo({ .x = 1, .y = 2 }); }",
    ).is_ok());
}

#[test]
fn enum_init_in_struct_field() {
    assert!(parse_and_type_check(
        "enum Inner { int V; None; }\nstruct Outer { enum Inner field; }\nvoid main() { struct Outer o = { .field = { .V = 42 } }; }",
    ).is_ok());
}

#[test]
fn enum_init_in_call() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid foo(enum Option x) { }\nvoid main() { foo({ .Some = 42 }); }",
    ).is_ok());
}

#[test]
fn struct_init_in_call_type_mismatch() {
    assert!(parse_and_type_check(
        "struct Point { int x; }\nstruct Other { int y; }\nvoid foo(struct Point p) { }\nvoid main() { foo({ .y = 1 }); }",
    ).is_err());
}

#[test]
fn enum_init_in_call_type_mismatch() {
    assert!(parse_and_type_check(
        "enum A { int X; }\nenum B { int Y; }\nvoid foo(enum A a) { }\nvoid main() { foo({ .Y = 1 }); }",
    ).is_err());
}

#[test]
fn non_exhaustive_match_rejected() {
    let result = parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .Some = 1 }; match x { case Some: { int y = Some; } } }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("non-exhaustive match"));
}

#[test]
fn duplicate_match_arm_rejected() {
    let result = parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .Some = 1 }; match x { case Some: { int y = Some; } case Some: { int z = Some; } case None: { } } }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("duplicate match arm"));
}

#[test]
fn unknown_match_variant_rejected() {
    let result = parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .Some = 1 }; match x { case Other: { } case None: { } } }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown variant"));
}

#[test]
fn unknown_udt_in_struct_field_rejected() {
    let result = parse_and_type_check(
        "struct Box { struct Missing value; }\nvoid main() { int x = 0; }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown struct type"));
}

#[test]
fn unknown_udt_in_enum_payload_rejected() {
    let result = parse_and_type_check(
        "enum Box { struct Missing Value; None; }\nvoid main() { int x = 0; }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown struct type"));
}

#[test]
fn unknown_udt_in_function_parameter_rejected() {
    let result = parse_and_type_check(
        "void take(struct Missing x) { }\nvoid main() { int x = 0; }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown struct type"));
}

#[test]
fn unknown_udt_in_function_return_rejected() {
    let result = parse_and_type_check(
        "struct Missing make() { return { .x = 0 }; }\nvoid main() { int x = 0; }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown struct type"));
}

#[test]
fn nested_unknown_udt_rejected() {
    let result = parse_and_type_check(
        "struct Box { struct Missing[] values; }\nvoid main() { int x = 0; }",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("unknown struct type"));
}

#[test]
fn invalid_scalar_to_struct_cast_rejected() {
    assert!(parse_and_type_check(
        "struct Point { int x; }\nvoid main() { struct Point p = (struct Point)42; }",
    )
    .is_err());
}

#[test]
fn invalid_scalar_to_enum_cast_rejected() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = (enum Option)42; }",
    )
    .is_err());
}

#[test]
fn invalid_struct_to_scalar_cast_rejected() {
    assert!(parse_and_type_check(
        "struct Point { int x; }\nvoid main() { struct Point p = { .x = 1 }; int y = (int)p; }",
    )
    .is_err());
}

#[test]
fn invalid_enum_to_scalar_cast_rejected() {
    assert!(parse_and_type_check(
        "enum Option { int Some; None; }\nvoid main() { enum Option x = { .Some = 1 }; float y = (float)x; }",
    )
    .is_err());
}

#[test]
fn invalid_unrelated_udt_cast_rejected() {
    assert!(parse_and_type_check(
        "struct Point { int x; }\nstruct Other { int y; }\nvoid main() { struct Other o = { .y = 1 }; struct Point p = (struct Point)o; }",
    )
    .is_err());
}

#[test]
fn numeric_casts_still_work() {
    assert!(parse_and_type_check("void main() { int x = (int)1.5; float y = (float)x; }").is_ok());
}

#[test]
fn contextual_udt_initializer_assignment_and_return_work() {
    assert!(parse_and_type_check(
        "struct Point { int x; }\nstruct Point make() { return { .x = 1 }; }\nvoid main() { struct Point p = { .x = 0 }; p = { .x = 2 }; }",
    )
    .is_ok());
}
