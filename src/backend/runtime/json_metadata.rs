// Owns generated ext/json constant and error metadata.
// Keep this chunk ordered by `super::RUNTIME_C`; later runtime chunks depend on it.

pub(super) const C: &str = include_str!("json_metadata.c");
