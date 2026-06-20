mod arrays;
mod core_values;
mod csv;
mod diagnostics;
mod internals;
mod numeric_comparison;
mod query;
mod strings;

use std::sync::OnceLock;

static RUNTIME_C: OnceLock<String> = OnceLock::new();

// Ownership map for generated C runtime chunks:
// - core_values: headers, boxed value types, constructors, and shared allocation helpers.
// - arrays: ordered-array storage, key canonicalization, iteration, offset reads, and array comparisons.
// - diagnostics: diagnostic sink setup and generic warning/fatal emission helpers.
// - numeric_comparison: numeric conversion, scalar casts, truthiness, comparisons, arithmetic, bitwise, and shifts.
// - strings: scalar string conversion, concatenation, type predicates, constants, float formatting, and echo output.
// - query: query string encoding and parse_str parity helpers used by internals/request setup.
// - csv: fgetcsv/fputcsv/str_getcsv parser and writer parity helpers.
// - internals: runtime symbol tables plus optional internal-function handlers and dispatch.
pub(super) fn runtime_c() -> &'static str {
    RUNTIME_C.get_or_init(|| {
        let internal_functions_c = internals::internal_functions_c();
        let chunks = [
            core_values::C,
            arrays::STORAGE_C,
            internals::SYMBOLS_C,
            diagnostics::C,
            numeric_comparison::CONVERSION_AND_COMPARISON_PROLOGUE_C,
            arrays::ACCESS_AND_ITERATION_C,
            numeric_comparison::COMPARISON_AND_OPERATORS_C,
            strings::C,
        ];
        let len =
            chunks.iter().map(|chunk| chunk.len()).sum::<usize>() + internal_functions_c.len();
        let mut runtime = String::with_capacity(len);
        for chunk in chunks {
            runtime.push_str(chunk);
        }
        runtime.push_str(&internal_functions_c);
        runtime
    })
}

pub(super) const INTERNAL_FUNCTIONS_START: &str = "/* PTN_INTERNAL_FUNCTIONS_START */";
pub(super) const INTERNAL_FUNCTIONS_END: &str = "/* PTN_INTERNAL_FUNCTIONS_END */";
pub(super) const QUERY_RUNTIME_MODULE: &str = "/* PTN_QUERY_RUNTIME_MODULE */";
pub(super) const DIRECT_INTERNAL_HELPERS_START: &str = "/* PTN_DIRECT_INTERNAL_HELPERS_START */";
pub(super) const DIRECT_INTERNAL_HELPERS_END: &str = "/* PTN_DIRECT_INTERNAL_HELPERS_END */";
pub(super) const COMPACT_INTERNAL_HELPERS_START: &str = "/* PTN_COMPACT_INTERNAL_HELPERS_START */";
pub(super) const COMPACT_INTERNAL_HELPERS_END: &str = "/* PTN_COMPACT_INTERNAL_HELPERS_END */";
