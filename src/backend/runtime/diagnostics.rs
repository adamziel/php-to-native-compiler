// Owns the generated C runtime diagnostics lane.
// Keep these chunks ordered by `super::RUNTIME_C`; C declaration order is part of the ABI.

pub(super) const C: &str = include_str!("diagnostics.c");
