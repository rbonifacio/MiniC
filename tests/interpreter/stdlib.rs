use super::helpers::run;

#[test]
fn print_int() {
    assert!(run("void main() { print(42); }").is_ok());
}

#[test]
fn print_bool() {
    assert!(run("void main() { print(true); }").is_ok());
}

#[test]
fn print_array() {
    assert!(run("void main() { print([1, 2, 3]); }").is_ok());
}

#[test]
fn stdlib_sqrt_int_coercion() {
    assert!(run("void main() { float r = sqrt(4); }").is_ok());
}

#[test]
fn stdlib_pow_int_args() {
    assert!(run("void main() { float r = pow(2, 10); }").is_ok());
}

#[test]
fn stdlib_read_fns_type_check() {
    assert!(run("void main() { if false { int x = readInt(); } if false { float x = readFloat(); } if false { str x = readString(); } }").is_ok());
}

#[test]
fn stdlib_pow_float_args() {
    assert!(run("void main() { float r = pow(2.0, 3.0); }").is_ok());
}
