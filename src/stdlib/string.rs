//! String built-in functions for MiniC: `len`, `substr`, `toUpper`, `toLower`, `strToInt`, `strToFloat`, `contains`.
//!
//! # Overview
//!
//! Exposes seven public functions, each matching the [`crate::interpreter::value::NativeFn`] signature
//! `fn(Vec<Value>) -> Result<Value, RuntimeError>`:
//!
//! * [`substr`] — returns a substring of a string.
//! * [`toUpper`] — converts a string to uppercase.
//! * [`toLower`] — converts a string to lowercase.
//! * [`strToInt`] — converts a string to an integer.
//! * [`strToFloat`] — converts a string to a float.
//!

use crate::interpreter::value::{RuntimeError, Value};


pub fn substr(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::new(format!(
            "substr expects 3 arguments, got {}",
            args.len()
        )));
    }
    let s = match &args[0] {
        Value::Str(s) => s,
        v => {
            return Err(RuntimeError::new(format!(
                "substr: expected string argument, got {}",
                v
            )))
        }
    };
    let start = match &args[1] {
        Value::Int(n) => *n,
        v => {
            return Err(RuntimeError::new(format!(
                "substr: expected int start index, got {}",
                v
            )))
        }
    };
    let length = match &args[2] {
        Value::Int(n) => *n,
        v => {
            return Err(RuntimeError::new(format!(
                "substr: expected int length, got {}",
                v
            )))
        }
    };
    if start < 0 {
        return Err(RuntimeError::new(format!(
            "substr: start index out of bounds: {}",
            start
        )));
    }
    if length < 0 {
        return Err(RuntimeError::new(format!(
            "substr: length out of bounds: {}",
            length
        )));
    }

    let chars: Vec<char> = s.chars().collect();
    let start = start as usize;
    let length = length as usize;

    if start > chars.len() {
        return Err(RuntimeError::new(format!(
            "substr: start index out of bounds: {} (len: {})",
            start,
            chars.len()
        )));
    }

    let end = start.checked_add(length).ok_or_else(|| {
        RuntimeError::new(format!(
            "substr: range overflow for start {} and length {}",
            start, length
        ))
    })?;

    if end > chars.len() {
        return Err(RuntimeError::new(format!(
            "substr: range out of bounds: [{}..{}) for len {}",
            start,
            end,
            chars.len()
        )));
    }

    Ok(Value::Str(chars[start..end].iter().collect()))
}

pub fn to_upper(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(format!(
            "toUpper expects 1 argument, got {}",
            args.len()
        )));
    }
    match &args[0] {
        Value::Str(s) => Ok(Value::Str(s.to_uppercase())),
        v => Err(RuntimeError::new(format!(
            "toUpper: expected string argument, got {}",
            v
        ))),
    }
}

pub fn to_lower(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(format!(
            "toLower expects 1 argument, got {}",
            args.len()
        )));
    }
    match &args[0] {
        Value::Str(s) => Ok(Value::Str(s.to_lowercase())),
        v => Err(RuntimeError::new(format!(
            "toLower: expected string argument, got {}",
            v
        ))),
    }
}

pub fn str_to_int(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(format!(
            "strToInt expects 1 argument, got {}",
            args.len()
        )));
    }
    match &args[0] {
        Value::Str(s) => match s.trim().parse::<i64>() {
            Ok(n) => Ok(Value::Int(n)),
            Err(_) => Err(RuntimeError::new(format!(
                "strToInt: cannot convert '{}' to int",
                s
            ))),
        },
        v => Err(RuntimeError::new(format!(
            "strToInt: expected string argument, got {}",
            v
        ))),
    }
}

pub fn str_to_float(args: Vec<Value>) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::new(format!(
            "strToFloat expects 1 argument, got {}",
            args.len()
        )));
    }
    match &args[0] {
        Value::Str(s) => match s.trim().parse::<f64>() {
            Ok(x) => Ok(Value::Float(x)),
            Err(_) => Err(RuntimeError::new(format!(
                "strToFloat: cannot convert '{}' to float",
                s
            ))),
        },
        v => Err(RuntimeError::new(format!(
            "strToFloat: expected string argument, got {}",
            v
        ))),
    }
}
