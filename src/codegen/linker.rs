use std::collections::HashSet;
use crate::stdlib::NativeRegistry;

/// The Linker holds the set of function names that are implemented externally
/// (i.e., native stdlib functions). It is built from a [`NativeRegistry`] and
/// used during TAC generation to distinguish `Call` from `ExternCall`.
pub struct Linker {
    extern_names: HashSet<String>,
}

impl Linker {
    /// Build a linker from the default native registry.
    /// All names registered in the stdlib become extern symbols.
    pub fn new() -> Self {
        let registry = NativeRegistry::default();
        let extern_names = registry.iter()
            .map(|(name, _)| name.clone())
            .collect();
        Self { extern_names }
    }

    /// Returns `true` if `name` refers to a native (stdlib) function.
    /// The code generator uses this to emit `ExternCall` instead of `Call`.
    pub fn is_extern(&self, name: &str) -> bool {
        self.extern_names.contains(name)
    }

    /// Returns the full set of extern names. Used by the code generation
    /// environment to carry linker knowledge through the translation.
    pub fn extern_names(&self) -> &HashSet<String> {
        &self.extern_names
    }
}
