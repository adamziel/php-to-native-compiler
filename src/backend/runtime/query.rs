// Owns query string encoding and parse_str parity helpers.
// This chunk is spliced into the internals lane after generic argument helpers.

pub(super) const C: &str = include_str!("query.c");
