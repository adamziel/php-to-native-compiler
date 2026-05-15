# mysqli Extension

Status: boundary only.

`mysqli_connect`, `mysqli_real_connect`, `mysqli_get_server_info`,
`mysqli_query`, `mysqli_select_db`, `mysqli_real_escape_string`,
`mysqli_fetch_object`, `mysqli_fetch_assoc`, `mysqli_fetch_field`,
`mysqli_num_fields`, `mysqli_free_result`, `mysqli_more_results`,
`mysqli_next_result`, `mysqli_report`, and `mysqli_init` are currently visible through
`function_exists()`, `is_callable()`, dynamic string-valued function lookup,
and native function-table introspection so WordPress' early database startup
paths can move to the next real bootstrap blocker.

`mysqli_report($mode)` accepts the current WordPress startup mode
`MYSQLI_REPORT_OFF` and the common PHP 8 strict mode combination
`MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT`, records the mode, and returns
`true`. Report-mode warning/error routing is not implemented.

`mysqli_init()` returns a placeholder `mysqli` object with `connect_errno = 0`
and `connect_error = null`. It is an object-shape compatibility boundary, not a
host connection or real mysqli handle.

`mysqli_real_connect($handle, ...)` accepts the current WordPress startup call
shape for that placeholder object, writes `connect_errno = 0` and
`connect_error = null`, and returns `true`. It is deterministic fake success
for bootstrap exploration only. It does not open sockets, authenticate,
select a database, negotiate charset, or produce real mysqli connection state.

`mysqli_get_server_info($handle)` accepts the placeholder object and returns
`8.0.0-phpc-placeholder`, a deterministic fake server string used by
WordPress' `wpdb::db_version()` guard. It does not query a server or reflect
real connection metadata.

`mysqli_query($handle, 'SELECT @@SESSION.sql_mode')` accepts the placeholder
object and that exact SQL mode probe, returning `false` as a deterministic
empty/no-result boundary. This lets WordPress skip SQL mode normalization
without executing SQL or producing a result resource.

`mysqli_query($handle, 'SELECT * FROM wp_posts WHERE 1 = 0')` returns a
placeholder `mysqli_result` object for the first deterministic empty result
lifecycle boundary. `mysqli_num_fields($result)` returns `0`,
`mysqli_fetch_field($result)` and `mysqli_fetch_object($result)` return
`false`, and `mysqli_free_result($result)` returns `null`. For the placeholder
connection, `mysqli_more_results($handle)` and `mysqli_next_result($handle)`
return `false`. This does not execute SQL, store rows, expose real field
metadata, or model real result resources.

`mysqli_query($handle, 'SELECT ID, post_title FROM wp_posts WHERE ID = 1')`
returns a placeholder `mysqli_result` object with deterministic interpreter
state: fields `ID` and `post_title`, plus one row where `ID` is `1` and
`post_title` is `Hello world placeholder`. `mysqli_num_fields()` returns `2`,
`mysqli_fetch_field()` returns `stdClass` objects with a `name` property for
those two fields and then `false`, and `mysqli_fetch_object()` returns one
`stdClass` row object and then `false`. `mysqli_fetch_assoc()` uses the same
row cursor and returns one associative PHP array with keys `ID` and
`post_title` and then `false`. This is a fixed row-shape and cursor boundary,
not SQL execution, database storage, WordPress content fidelity, broad fetch
mode support, or real mysqli metadata.

Other `SELECT` statements are rejected with a specific non-empty-result-set
diagnostic. For example, `SELECT 1` reports that
non-empty `mysqli` result sets are not implemented. This is an explicit
compatibility boundary before general row storage, SQL parsing/execution,
database-backed WordPress queries, and real metadata exist.

`mysqli_select_db($handle, $database)` accepts the placeholder object and a
string or null database name, returning deterministic `true`. It does not
select or validate a real database.

Calling `mysqli_connect(...)` is still a stable unsupported runtime boundary:

```text
unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset
```

No real mysqli extension behavior is implemented yet: no host connections, no
real resources or connected objects beyond the placeholder shapes, no real
server metadata, no query execution beyond the documented deterministic
queries, no real database selection beyond deterministic success, no general
non-empty result sets, no real row/field metadata, no charset handling, no
numeric or mixed fetch modes, no errors/warnings, no transactions, no
configuration beyond the current report-mode flag, no PDO bridge, and no
native database lowering.
