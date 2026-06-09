// Owns the generated C runtime numeric comparison lane.
// Keep these chunks ordered by `super::RUNTIME_C`; C declaration order is part of the ABI.

pub(super) const CONVERSION_AND_COMPARISON_PROLOGUE_C: &str =
    include_str!("numeric_conversion_and_comparison_prologue.c");
pub(super) const COMPARISON_AND_OPERATORS_C: &str =
    include_str!("numeric_comparison_and_operators.c");
