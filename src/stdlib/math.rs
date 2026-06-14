//! Math built-in functions for MiniC: `pow` and `sqrt`.
//!
//! # Overview
//!
//! Exposes two public functions, each matching the [`crate::interpreter::value::NativeFn`] signature
//! `fn(Vec<Value>) -> Result<Value, RuntimeError>`:
//!
//! * [`pow_fn`] — raises a base to an exponent: `pow(base, exp)`. Both
//!   arguments may be `int` or `float`; the result is always a `float`.
//! * [`sqrt_fn`] — computes the square root: `sqrt(x)`. The argument may be
//!   `int` or `float`; the result is always a `float`.
//!
//! Both functions accept `int` arguments through the private `to_float`
//! helper, which coerces `Value::Int` to `f64` before performing the
//! calculation. This matches the MiniC type rule that arithmetic involving
//! floats always produces a float.

use crate::interpreter::value::{RuntimeError, Value};

pub fn pow_fn(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::new(format!(
            "pow expects 2 arguments, got {}",
            args.len()
        )));
    }
    let base = to_float(&args[0], "pow base")?;
    let exp = to_float(&args[1], "pow exponent")?;
    Ok(Value::Float(base.powf(exp)))
}

pub fn sqrt_fn(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(format!(
            "sqrt expects 1 argument, got {}",
            args.len()
        )));
    }
    let x = to_float(&args[0], "sqrt argument")?;
    Ok(Value::Float(x.sqrt()))
}

fn to_float(val: &Value, context: &str) -> Result<f64, RuntimeError> {
    match val {
        Value::Int(n) => Ok(*n as f64),
        Value::Float(x) => Ok(*x),
        v => Err(RuntimeError::new(format!(
            "{}: expected numeric value, got {}",
            context, v
        ))),
    }
}
