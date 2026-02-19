//! Parser module for MiniC language.

pub mod expressions;
pub mod functions;
pub mod identifiers;
pub mod literals;
pub mod program;
pub mod statements;

pub use expressions::expression;
pub use functions::fun_decl;
pub use identifiers::identifier;
pub use literals::{literal, Literal};
pub use program::program;
pub use statements::{assignment, statement};
