use mini_c::interpreter::value::Value;
use mini_c::ir::ast::Type;
use mini_c::stdlib::io::print_fn;
use mini_c::stdlib::math::{pow_fn, sqrt_fn};
use mini_c::stdlib::string::substr;
use mini_c::stdlib::NativeRegistry;

// --- io tests ---

#[test]
fn test_print_fn_integer() {
    let result = print_fn(vec![Value::Int(42)]);
    assert_eq!(result, Ok(Value::Void));
}

#[test]
fn test_print_fn_bool() {
    let result = print_fn(vec![Value::Bool(true)]);
    assert_eq!(result, Ok(Value::Void));
}

#[test]
fn test_print_fn_array() {
    let result = print_fn(vec![Value::Array(vec![Value::Int(1), Value::Int(2)])]);
    assert_eq!(result, Ok(Value::Void));
}

#[test]
fn test_print_fn_no_args() {
    let result = print_fn(vec![]);
    assert_eq!(result, Ok(Value::Void));
}

// --- math tests ---

#[test]
fn test_pow_int_args() {
    let result = pow_fn(vec![Value::Int(2), Value::Int(10)]);
    assert_eq!(result, Ok(Value::Float(1024.0)));
}

#[test]
fn test_pow_float_args() {
    let result = pow_fn(vec![Value::Float(2.0), Value::Float(0.5)]);
    match result {
        Ok(Value::Float(v)) => assert!((v - 1.4142135).abs() < 1e-5),
        _ => panic!("expected Float"),
    }
}

#[test]
fn test_pow_negative_exponent() {
    let result = pow_fn(vec![Value::Float(2.0), Value::Float(-1.0)]);
    assert_eq!(result, Ok(Value::Float(0.5)));
}

#[test]
fn test_pow_wrong_arity() {
    let result = pow_fn(vec![Value::Float(2.0)]);
    assert!(result.is_err());
}

#[test]
fn test_sqrt_perfect_square() {
    let result = sqrt_fn(vec![Value::Int(4)]);
    assert_eq!(result, Ok(Value::Float(2.0)));
}

#[test]
fn test_sqrt_float() {
    let result = sqrt_fn(vec![Value::Float(2.0)]);
    match result {
        Ok(Value::Float(v)) => assert!((v - 1.4142135).abs() < 1e-5),
        _ => panic!("expected Float"),
    }
}

#[test]
fn test_sqrt_zero() {
    let result = sqrt_fn(vec![Value::Int(0)]);
    assert_eq!(result, Ok(Value::Float(0.0)));
}

#[test]
fn test_sqrt_wrong_type() {
    let result = sqrt_fn(vec![Value::Bool(true)]);
    assert!(result.is_err());
}

// --- string tests ---

#[test]
fn test_substr_valid_slice() {
    let result = substr(vec![
        Value::Str("abcdef".to_string()),
        Value::Int(2),
        Value::Int(3),
    ]);
    assert_eq!(result, Ok(Value::Str("cde".to_string())));
}

#[test]
fn test_substr_start_out_of_bounds() {
    let result = substr(vec![
        Value::Str("abc".to_string()),
        Value::Int(4),
        Value::Int(1),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_substr_range_out_of_bounds() {
    let result = substr(vec![
        Value::Str("abc".to_string()),
        Value::Int(2),
        Value::Int(2),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_substr_negative_start_rejected() {
    let result = substr(vec![
        Value::Str("abc".to_string()),
        Value::Int(-1),
        Value::Int(1),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_substr_negative_length_rejected() {
    let result = substr(vec![
        Value::Str("abc".to_string()),
        Value::Int(0),
        Value::Int(-1),
    ]);
    assert!(result.is_err());
}

// --- registry tests ---

#[test]
fn test_default_registry_contains_all_stdlib() {
    let r = NativeRegistry::default();
    assert!(r.lookup("print").is_some());
    assert!(r.lookup("readInt").is_some());
    assert!(r.lookup("readFloat").is_some());
    assert!(r.lookup("readString").is_some());
    assert!(r.lookup("pow").is_some());
    assert!(r.lookup("sqrt").is_some());
    assert!(r.lookup("substr").is_some());
    assert!(r.lookup("toUpper").is_some());
    assert!(r.lookup("toLower").is_some());
    assert!(r.lookup("strToInt").is_some());
    assert!(r.lookup("strToFloat").is_some());
}

#[test]
fn test_len_and_contains_not_in_registry() {
    let r = NativeRegistry::default();
    assert!(r.lookup("len").is_none());
    assert!(r.lookup("contains").is_none());
}

#[test]
fn test_lookup_unregistered_returns_none() {
    let r = NativeRegistry::default();
    assert!(r.lookup("unknown").is_none());
}

#[test]
fn test_sqrt_entry_signature() {
    let r = NativeRegistry::default();
    let entry = r.lookup("sqrt").unwrap();
    assert_eq!(entry.params, vec![Type::Float]);
    assert_eq!(entry.return_type, Type::Float);
}

#[test]
fn test_print_uses_type_any() {
    let r = NativeRegistry::default();
    let entry = r.lookup("print").unwrap();
    assert_eq!(entry.params, vec![Type::Any]);
}
