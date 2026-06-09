// Owns the generated C runtime arrays lane.
// Keep these chunks ordered by `super::RUNTIME_C`; C declaration order is part of the ABI.

pub(super) const STORAGE_C: &str = include_str!("arrays_storage.c");
pub(super) const ACCESS_AND_ITERATION_C: &str = include_str!("arrays_access_and_iteration.c");
