mod arrays;
mod core_values;
mod diagnostics;
mod internals;
mod numeric_comparison;
mod strings;

use std::sync::OnceLock;

static RUNTIME_C: OnceLock<String> = OnceLock::new();

// Ownership map for generated C runtime chunks:
// - core_values: headers, boxed value types, constructors, and shared allocation helpers.
// - arrays: ordered-array storage, key canonicalization, iteration, offset reads, and array comparisons.
// - diagnostics: diagnostic sink setup and generic warning/fatal emission helpers.
// - numeric_comparison: numeric conversion, scalar casts, truthiness, comparisons, arithmetic, bitwise, and shifts.
// - strings: scalar string conversion, concatenation, type predicates, constants, float formatting, and echo output.
// - internals: runtime symbol tables plus optional internal-function handlers and dispatch.
pub(super) fn runtime_c() -> &'static str {
    RUNTIME_C.get_or_init(|| {
        let chunks = [
            core_values::C,
            arrays::STORAGE_C,
            internals::SYMBOLS_C,
            diagnostics::C,
            numeric_comparison::CONVERSION_AND_COMPARISON_PROLOGUE_C,
            arrays::ACCESS_AND_ITERATION_C,
            numeric_comparison::COMPARISON_AND_OPERATORS_C,
            strings::C,
            internals::INTERNAL_FUNCTIONS_C,
        ];
        let len = chunks.iter().map(|chunk| chunk.len()).sum();
        let mut runtime = String::with_capacity(len);
        for chunk in chunks {
            runtime.push_str(chunk);
        }
        runtime
    })
}

pub(super) const INTERNAL_FUNCTIONS_START: &str = "/* PTN_INTERNAL_FUNCTIONS_START */";
pub(super) const INTERNAL_FUNCTIONS_END: &str = "/* PTN_INTERNAL_FUNCTIONS_END */";
