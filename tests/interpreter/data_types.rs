use super::helpers::run;

#[test]
fn struct_member_assign_and_read() {
    assert!(run("struct Point { int x; int y; } void main() { struct Point p = { .x = 21, .y = 0 }; int v = p.x; }").is_ok());
}

#[test]
fn enum_init_and_match() {
    assert!(run("enum Option { int Some; None; } void main() { enum Option x = { .Some = 42 }; int result = 0; match x { case Some: { result = Some; } case None: { result = -1; } } }").is_ok());
}

#[test]
fn enum_match_unit_variant() {
    assert!(run("enum Option { int Some; None; } void main() { enum Option x = { .None }; int result = -1; match x { case Some: { result = Some; } case None: { result = 99; } } }").is_ok());
}

#[test]
fn struct_init_values_preserved() {
    let result = run("struct Point { int x; int y; } void main() { struct Point p = { .x = 0, .y = 10 }; int[] arr = [1]; int v = arr[p.y]; }");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn struct_value_equality() {
    assert!(run("struct Pair { int a; int b; } void main() { struct Pair p = { .a = 1, .b = 2 }; struct Pair q = { .a = 1, .b = 2 }; bool eq = p == q; }").is_ok());
}

#[test]
fn enum_value_equality() {
    assert!(run("enum Option { int Some; None; } void main() { enum Option a = { .Some = 1 }; enum Option b = { .Some = 1 }; bool eq = a == b; }").is_ok());
}

#[test]
fn match_binding_does_not_leak() {
    assert!(run("enum Option { int Some; None; } void main() { int Some = 0; enum Option x = { .Some = 42 }; match x { case Some: { int y = Some; } case None: { } } int[] arr = [1]; int v = arr[Some]; }").is_ok());
}
