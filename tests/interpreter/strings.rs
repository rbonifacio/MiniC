use super::helpers::run;

#[test]
fn string_comparison() {
    assert!(run("void main() { str a = \"hello\"; str b = \"hello\"; bool eq = a == b; }").is_ok());
}

#[test]
fn string_inequality() {
    assert!(run("void main() { str a = \"abc\"; str b = \"xyz\"; bool eq = a == b; }").is_ok());
}

#[test]
fn string_default_value() {
    assert!(run("void main() { str s = \"\"; }").is_ok());
}
