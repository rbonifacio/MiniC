use mini_c::interpreter::value::Value;
use mini_c::ir::ast::Type;
use mini_c::stdlib::io::print_fn;
use mini_c::stdlib::math::{pow_fn, sqrt_fn};
use mini_c::stdlib::NativeRegistry;

// --- io ---

#[test]
fn print_fn_integer() {
    assert_eq!(print_fn(vec![Value::Int(42)]), Ok(Value::Void));
}

#[test]
fn print_fn_bool() {
    assert_eq!(print_fn(vec![Value::Bool(true)]), Ok(Value::Void));
}

#[test]
fn print_fn_array() {
    assert_eq!(
        print_fn(vec![Value::Array(vec![Value::Int(1), Value::Int(2)])]),
        Ok(Value::Void)
    );
}

#[test]
fn print_fn_no_args() {
    assert_eq!(print_fn(vec![]), Ok(Value::Void));
}

// --- math ---

#[test]
fn pow_int_args() {
    assert_eq!(
        pow_fn(vec![Value::Int(2), Value::Int(10)]),
        Ok(Value::Float(1024.0))
    );
}

#[test]
fn pow_float_args() {
    let result = pow_fn(vec![Value::Float(2.0), Value::Float(0.5)]);
    match result {
        Ok(Value::Float(v)) => assert!((v - 1.4142135).abs() < 1e-5),
        _ => panic!("expected Float"),
    }
}

#[test]
fn pow_negative_exponent() {
    assert_eq!(
        pow_fn(vec![Value::Float(2.0), Value::Float(-1.0)]),
        Ok(Value::Float(0.5))
    );
}

#[test]
fn pow_wrong_arity() {
    assert!(pow_fn(vec![Value::Float(2.0)]).is_err());
}

#[test]
fn sqrt_perfect_square() {
    assert_eq!(sqrt_fn(vec![Value::Int(4)]), Ok(Value::Float(2.0)));
}

#[test]
fn sqrt_float() {
    let result = sqrt_fn(vec![Value::Float(2.0)]);
    match result {
        Ok(Value::Float(v)) => assert!((v - 1.4142135).abs() < 1e-5),
        _ => panic!("expected Float"),
    }
}

#[test]
fn sqrt_zero() {
    assert_eq!(sqrt_fn(vec![Value::Int(0)]), Ok(Value::Float(0.0)));
}

#[test]
fn sqrt_wrong_type() {
    assert!(sqrt_fn(vec![Value::Bool(true)]).is_err());
}

// --- registry ---

#[test]
fn default_registry_contains_all_stdlib() {
    let r = NativeRegistry::default();
    assert!(r.lookup("print").is_some());
    assert!(r.lookup("readInt").is_some());
    assert!(r.lookup("readFloat").is_some());
    assert!(r.lookup("readString").is_some());
    assert!(r.lookup("pow").is_some());
    assert!(r.lookup("sqrt").is_some());
}

#[test]
fn lookup_unregistered_returns_none() {
    let r = NativeRegistry::default();
    assert!(r.lookup("unknown").is_none());
}

#[test]
fn sqrt_entry_signature() {
    let r = NativeRegistry::default();
    let entry = r.lookup("sqrt").unwrap();
    assert_eq!(entry.params, vec![Type::Float]);
    assert_eq!(entry.return_type, Type::Float);
}

#[test]
fn print_uses_type_any() {
    let r = NativeRegistry::default();
    let entry = r.lookup("print").unwrap();
    assert_eq!(entry.params, vec![Type::Any]);
}
