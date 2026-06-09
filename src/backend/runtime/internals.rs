// Owns the generated C runtime internals lane.
// Keep these chunks ordered by `super::RUNTIME_C`; C declaration order is part of the ABI.

pub(super) const SYMBOLS_C: &str = include_str!("internals_symbols.c");
pub(super) const INTERNAL_FUNCTIONS_C: &str = include_str!("internals_internal_functions.c");
