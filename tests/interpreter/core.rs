use super::helpers::run;

#[test]
fn empty_main() {
    assert!(run("void main() {}").is_ok());
}

#[test]
fn arithmetic_int() {
    assert!(
        run("int add(int a, int b) { return a + b; } void main() { int r = add(3, 4); }").is_ok()
    );
}

#[test]
fn arithmetic_float_coercion() {
    assert!(run("float f() { return 2 + 1.5; } void main() { float r = f(); }").is_ok());
}

#[test]
fn if_true_branch() {
    assert!(run("int choose(bool cond) { if cond { return 1; } else { return 2; } return 0; } void main() { int r = choose(true); }").is_ok());
}

#[test]
fn if_false_branch() {
    assert!(run("int choose(bool cond) { if cond { return 1; } else { return 2; } return 0; } void main() { int r = choose(false); }").is_ok());
}

#[test]
fn nested_if_else_chain() {
    let src = "void main() { int x = 2; int r = 0; if x == 1 { r = 10; } else { if x == 2 { r = 20; } else { r = 30; } } }";
    assert!(run(src).is_ok());
}

#[test]
fn while_loop() {
    assert!(run("int count_to(int n) { int i = 0; while i < n { i = i + 1; } return i; } void main() { int r = count_to(3); }").is_ok());
}

#[test]
fn while_no_iteration() {
    assert!(run("void main() { int x = 0; while false { x = 1; } }").is_ok());
}

#[test]
fn factorial() {
    assert!(run("int factorial(int n) { if n <= 1 { return 1; } return n * factorial(n - 1); } void main() { int r = factorial(5); }").is_ok());
}

#[test]
fn undefined_function() {
    assert!(run("void main() { foo(1); }").is_err());
}
