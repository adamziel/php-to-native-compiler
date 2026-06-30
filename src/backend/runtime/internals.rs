// Owns the generated C runtime internals lane.
// Keep these chunks ordered by `super::RUNTIME_C`; C declaration order is part of the ABI.

pub(super) const SYMBOLS_C: &str = include_str!("internals_symbols.c");
pub(super) const INTERNAL_FUNCTIONS_C: &str = include_str!("internals_internal_functions.c");
pub(super) const CRYPT_PORT_C: &str = include_str!("crypt_port.c");
pub(super) const HASH_SNEFRU_TABLES_C: &str = include_str!("hash_snefru_tables.c");
pub(super) const HASH_LEGACY_EXTRA_C: &str = include_str!("hash_legacy_extra.c");
pub(super) const HASH_EXTRA_C: &str = include_str!("hash_extra.c");
pub(super) const BCMATH_CALENDAR_C: &str = include_str!("bcmath_calendar.c");

const CSV_RUNTIME_MARKER: &str = "/* PTN_CSV_RUNTIME_START */";

pub(super) fn internal_functions_c() -> String {
    let marker = super::INTERNAL_FUNCTIONS_START;
    let marker_start = INTERNAL_FUNCTIONS_C
        .find(marker)
        .expect("internal-functions start marker should exist");
    let marker_end = marker_start + marker.len();
    let csv_marker_start = INTERNAL_FUNCTIONS_C
        .find(CSV_RUNTIME_MARKER)
        .expect("csv runtime marker should exist");
    assert!(
        csv_marker_start >= marker_end,
        "csv runtime marker should follow internal-functions start marker"
    );
    let csv_marker_end = csv_marker_start + CSV_RUNTIME_MARKER.len();
    let query_marker = super::QUERY_RUNTIME_MODULE;
    let mut source = String::with_capacity(
        INTERNAL_FUNCTIONS_C.len()
            + CRYPT_PORT_C.len()
            + HASH_SNEFRU_TABLES_C.len()
            + HASH_LEGACY_EXTRA_C.len()
            + HASH_EXTRA_C.len()
            + BCMATH_CALENDAR_C.len()
            + super::csv::C.len()
            + super::query::C.len()
            + 4,
    );
    source.push_str(&INTERNAL_FUNCTIONS_C[..marker_end]);
    source.push('\n');
    source.push_str(CRYPT_PORT_C);
    source.push('\n');
    source.push_str(HASH_SNEFRU_TABLES_C);
    source.push('\n');
    source.push_str(HASH_LEGACY_EXTRA_C);
    source.push('\n');
    source.push_str(HASH_EXTRA_C);
    source.push('\n');
    source.push_str(BCMATH_CALENDAR_C);
    source.push('\n');
    source.push_str(&INTERNAL_FUNCTIONS_C[marker_end..csv_marker_end]);
    source.push('\n');
    source.push_str(super::csv::C);
    source.push_str(&INTERNAL_FUNCTIONS_C[csv_marker_end..]);
    let query_marker_start = source
        .find(query_marker)
        .expect("query runtime module marker should exist");
    source.replace_range(
        query_marker_start..query_marker_start + query_marker.len(),
        super::query::C,
    );
    source
}
