//! IO built-in functions for MiniC: `print`, `readInt`, `readFloat`, `readString`.
//!
//! # Overview
//!
//! Exposes four public functions, each matching the [`crate::interpreter::value::NativeFn`] signature
//! `fn(Vec<Value>) -> Result<Value, RuntimeError>`:
//!
//! * [`print_fn`] — prints its argument (any value) to stdout followed by a
//!   newline. If called with no arguments, prints an empty line.
//! * [`read_int_fn`] — reads one line from stdin and parses it as an `i64`.
//! * [`read_float_fn`] — reads one line from stdin and parses it as an `f64`.
//! * [`read_string_fn`] — reads one line from stdin and returns it (trimmed)
//!   as a `str` value.
//!
//! All four are registered in [`NativeRegistry::default()`](super::NativeRegistry).

use std::io::{self, BufRead};

use crate::interpreter::value::{RuntimeError, Value};

pub fn print_fn(args: Vec<Value>) -> Result<Value, RuntimeError> {
    let val = args.into_iter().next().unwrap_or(Value::Void);
    println!("{}", val);
    Ok(Value::Void)
}

pub fn read_int_fn(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    let line = read_line()?;
    line.trim()
        .parse::<i64>()
        .map(Value::Int)
        .map_err(|e| RuntimeError::new(format!("readInt: invalid integer input: {}", e)))
}

pub fn read_float_fn(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    let line = read_line()?;
    line.trim()
        .parse::<f64>()
        .map(Value::Float)
        .map_err(|e| RuntimeError::new(format!("readFloat: invalid float input: {}", e)))
}

pub fn read_string_fn(_args: Vec<Value>) -> Result<Value, RuntimeError> {
    let line = read_line()?;
    Ok(Value::Str(line.trim().to_string()))
}

fn read_line() -> Result<String, RuntimeError> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| RuntimeError::new(format!("IO error: {}", e)))?;
    if line.is_empty() {
        return Err(RuntimeError::new("readString: unexpected EOF"));
    }
    Ok(line)
}
