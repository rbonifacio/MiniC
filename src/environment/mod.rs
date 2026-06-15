//! Variable and function binding environment for MiniC.
//!
//! # Overview
//!
//! This module provides [`Environment<V>`](env::Environment), a generic map
//! from names (`String`) to values of type `V`. It is used in two places:
//!
//! * The **type checker** instantiates it as `Environment<Type>`, storing the
//!   declared MiniC type of each in-scope name.
//! * The **interpreter** instantiates it as `Environment<Value>`, storing the
//!   runtime value of each in-scope name.
//!
//! Both functions (user-defined and native) are stored in the same
//! environment as variables — there is no separate function table.
//!
//! # Design Decisions
//!
//! See [`mod@env`] for the full design rationale, including the choice of a
//! single flat map over a scope stack, and the `snapshot`/`restore` mechanism
//! used for function calls and block scoping.

pub mod env;

pub use env::Environment;