use super::helpers::run;

#[test]
fn cast_coercion_float_to_int() {
    assert!(run("void main() { int x = (int)3.14; }").is_ok());
}

#[test]
fn cast_coercion_int_to_float() {
    assert!(run("void main() { float x = (float)3; }").is_ok());
}
