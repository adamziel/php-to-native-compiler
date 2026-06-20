// Owns the generated C runtime CSV parity lane.
// Keep this chunk inserted by `internals::internal_functions_c`; C declaration order is part of the ABI.

pub(super) const C: &str = include_str!("csv.c");
