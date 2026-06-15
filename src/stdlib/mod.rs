//! Standard library for MiniC: built-in functions and their registry.
//!
//! # Overview
//!
//! This module defines the infrastructure for MiniC's built-in functions and
//! provides the default set of standard library entries. It exposes:
//!
//! * [`NativeRegistry`] — a map from function name to [`NativeEntry`]. Both
//!   the type checker and the interpreter use this to look up stdlib
//!   functions.
//! * [`NativeEntry`] — bundles the MiniC type signature (parameter types +
//!   return type) with the Rust function that implements the behaviour.
//!
//! The default registry (via `NativeRegistry::default()`) registers:
//! `print`, `readInt`, `readFloat`, `readString` (IO), and `pow`, `sqrt`
//! (math). Implementations live in the [`io`] and [`math`] sub-modules.
//!
//! # Design Decisions
//!
//! ## Bundling type signature with implementation (`NativeEntry`)
//!
//! Each registry entry carries both the MiniC-level type information
//! (`params: Vec<Type>`, `return_type: Type`) and the Rust function pointer
//! (`func: NativeFn`) that does the actual work. This single registration
//! point is the *only* place where a native function needs to be declared;
//! both the type checker and the interpreter query the same registry, so
//! there is no risk of the type signature drifting out of sync with the
//! implementation.
//!
//! An alternative would have been separate type-signature tables and
//! function-pointer tables. That was rejected because duplicating the
//! registration increases maintenance cost.
//!
//! ## `print` uses `Type::Any`
//!
//! `print` is the one stdlib function that must accept arguments of any type
//! (`int`, `float`, `bool`, `str`, arrays). Rather than adding special-case
//! logic to the type checker, its parameter type is registered as
//! `Type::Any`. The type checker's `types_compatible` function already treats
//! `Any` as matching everything, so `print` gets polymorphic behaviour for
//! free without changing any type-checking rules.
//!
//! ## `NativeFn` as a function pointer
//!
//! `NativeFn` (defined in `interpreter::value`) is a *function pointer type*:
//! a value of this type holds the address of a Rust function with the
//! signature `fn(Vec<Value>) -> Result<Value, RuntimeError>`. Storing it in
//! `NativeEntry` means the registry owns a direct, lightweight reference to
//! the implementation — no heap allocation or dynamic dispatch needed.

use std::collections::HashMap;

use crate::ir::ast::Type;
use crate::interpreter::value::NativeFn;

pub mod io;
pub mod math;

/// A registry entry: MiniC type signature + Rust implementation.
pub struct NativeEntry {
    /// MiniC parameter types (used for arity and type checking).
    pub params: Vec<Type>,
    /// MiniC return type.
    pub return_type: Type,
    /// Rust implementation.
    pub func: NativeFn,
}

/// Maps function names to their native entries.
pub struct NativeRegistry {
    entries: HashMap<String, NativeEntry>,
}

impl NativeRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, entry: NativeEntry) {
        self.entries.insert(name.to_string(), entry);
    }

    pub fn lookup(&self, name: &str) -> Option<&NativeEntry> {
        self.entries.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &NativeEntry)> {
        self.entries.iter()
    }
}

impl Default for NativeRegistry {
    fn default() -> Self {
        let mut r = Self::new();

        // IO
        r.register("print", NativeEntry {
            params: vec![Type::Any],
            return_type: Type::Unit,
            func: io::print_fn,
        });
        r.register("readInt", NativeEntry {
            params: vec![],
            return_type: Type::Int,
            func: io::read_int_fn,
        });
        r.register("readFloat", NativeEntry {
            params: vec![],
            return_type: Type::Float,
            func: io::read_float_fn,
        });
        r.register("readString", NativeEntry {
            params: vec![],
            return_type: Type::Str,
            func: io::read_string_fn,
        });

        // Math
        r.register("pow", NativeEntry {
            params: vec![Type::Float, Type::Float],
            return_type: Type::Float,
            func: math::pow_fn,
        });
        r.register("sqrt", NativeEntry {
            params: vec![Type::Float],
            return_type: Type::Float,
            func: math::sqrt_fn,
        });

        r
    }
}
