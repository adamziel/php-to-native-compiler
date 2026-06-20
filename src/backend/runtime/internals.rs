// Owns the generated C runtime internals lane.
// Keep these chunks ordered by `super::RUNTIME_C`; C declaration order is part of the ABI.

pub(super) const SYMBOLS_C: &str = include_str!("internals_symbols.c");
pub(super) const INTERNAL_FUNCTIONS_C: &str = include_str!("internals_internal_functions.c");
pub(super) const HTML_TABLES_GENERATED_C: &str = include_str!("html_tables_generated.c");
pub(super) const CRYPT_PORT_C: &str = include_str!("crypt_port.c");
pub(super) const BCMATH_CALENDAR_C: &str = include_str!("bcmath_calendar.c");

pub(super) fn internal_functions_c() -> String {
    let marker = super::INTERNAL_FUNCTIONS_START;
    let marker_start = INTERNAL_FUNCTIONS_C
        .find(marker)
        .expect("internal-functions start marker should exist");
    let marker_end = marker_start + marker.len();
    let html_tables_marker = "/* PTN_HTML_TABLES_GENERATED_START */";
    let html_tables_marker_start = INTERNAL_FUNCTIONS_C
        .find(html_tables_marker)
        .expect("html tables generated marker should exist");
    let html_tables_marker_end = html_tables_marker_start + html_tables_marker.len();
    let mut source = String::with_capacity(
        INTERNAL_FUNCTIONS_C.len()
            + CRYPT_PORT_C.len()
            + BCMATH_CALENDAR_C.len()
            + HTML_TABLES_GENERATED_C.len()
            + 4,
    );
    source.push_str(&INTERNAL_FUNCTIONS_C[..marker_end]);
    source.push('\n');
    source.push_str(CRYPT_PORT_C);
    source.push('\n');
    source.push_str(BCMATH_CALENDAR_C);
    source.push('\n');
    source.push_str(&INTERNAL_FUNCTIONS_C[marker_end..html_tables_marker_end]);
    source.push('\n');
    source.push_str(HTML_TABLES_GENERATED_C);
    source.push('\n');
    source.push_str(&INTERNAL_FUNCTIONS_C[html_tables_marker_end..]);
    source
}
