//! The `Environment<V>` type: a name-to-value binding map.
//!
//! # Overview
//!
//! [`Environment<V>`] is a flat `HashMap` from `String` (a name) to `V` (a
//! value). It supports the four core operations needed by both the type
//! checker and the interpreter:
//!
//! * [`declare`](Environment::declare) — bind a new name (or overwrite).
//! * [`get`](Environment::get) — look up a name.
//! * [`set`](Environment::set) — update an existing binding.
//! * [`snapshot`](Environment::snapshot) / [`restore`](Environment::restore)
//!   — save and restore the entire map (used for scoping).
//!
//! Additionally, [`names`](Environment::names) and
//! [`remove_new`](Environment::remove_new) support block-exit cleanup.
//!
//! # Design Decisions
//!
//! ## A single generic struct serving two phases
//!
//! In Rust, `Environment<V>` is *generic* over `V` — the angle brackets mean
//! "this struct works for any type `V` you choose". The same struct is used
//! with `V = Type` in the type checker and `V = Value` in the interpreter.
//! This avoids duplicating identical HashMap logic in two places, and ensures
//! that both phases have the same scoping behaviour.
//!
//! An alternative would have been two separate structs (`TypeEnv` and
//! `ValueEnv`). That was rejected because the two environments really do have
//! identical structure — only the stored type differs.
//!
//! ## Flat map with snapshot/restore instead of a scope stack
//!
//! A common approach to scoping is a *stack of maps*: push a new map on
//! block entry, pop it on block exit. MiniC uses a different approach: a
//! single flat `HashMap` combined with full-clone snapshots.
//!
//! * **Function calls**: before a call, the entire map is cloned
//!   (`snapshot`). The callee's parameters and locals are declared directly
//!   in the map. After the call returns, the map is replaced with the saved
//!   clone (`restore`). This gives each function call a completely fresh
//!   scope while still being able to look up functions defined at the
//!   top level.
//!
//! * **Block statements**: on block entry, [`Environment::names`] records the set of
//!   currently bound names. On block exit, [`Environment::remove_new`] removes any name
//!   that was not in that set — i.e., any variable declared inside the block.
//!   Assignments to outer-scope variables are *not* undone, which is the
//!   correct behaviour (a loop counter updated inside a `while` body must
//!   persist after the body finishes).
//!
//! The flat-map approach is simpler to implement and reason about for a small
//! language. The cost — cloning the entire map on each function call — is
//! acceptable at MiniC's scale.

use std::collections::{HashMap, HashSet};

/// Unified parametric environment: maps names to values of type `V`.
/// Both variable bindings and function bindings are stored in the same map.
pub struct Environment<V> {
    bindings: HashMap<String, V>,
}

impl<V: Clone> Environment<V> {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Bind `name` to `value`, overwriting any existing binding.
    pub fn declare(&mut self, name: impl Into<String>, value: V) {
        self.bindings.insert(name.into(), value);
    }

    /// Look up a binding by name.
    pub fn get(&self, name: &str) -> Option<&V> {
        self.bindings.get(name)
    }

    /// Update an existing binding. Returns `false` if the name is not bound.
    pub fn set(&mut self, name: &str, value: V) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    /// Capture a full clone of the current bindings (for function call scoping).
    pub fn snapshot(&self) -> HashMap<String, V> {
        self.bindings.clone()
    }

    /// Replace all bindings with the given snapshot (for function call scoping).
    pub fn restore(&mut self, snapshot: HashMap<String, V>) {
        self.bindings = snapshot;
    }

    /// Return the set of currently bound names (for block-entry capture).
    pub fn names(&self) -> HashSet<String> {
        self.bindings.keys().cloned().collect()
    }

    /// Remove any binding whose name is not in `outer` (for block-exit cleanup).
    pub fn remove_new(&mut self, outer: &HashSet<String>) {
        self.bindings.retain(|k, _| outer.contains(k));
    }
}

impl<V: Clone> Default for Environment<V> {
    fn default() -> Self {
        Self::new()
    }
}
