# mysqli Extension

Status: boundary only.

`mysqli_connect`, `mysqli_real_connect`, `mysqli_get_server_info`,
`mysqli_set_charset`, `mysqli_query`, `mysqli_affected_rows`,
`mysqli_insert_id`, `mysqli_ping`, `mysqli_select_db`, `mysqli_real_escape_string`,
`mysqli_fetch_object`, `mysqli_fetch_assoc`, `mysqli_fetch_array`,
`mysqli_fetch_row`, `mysqli_fetch_field`, `mysqli_num_fields`,
`mysqli_num_rows`, `mysqli_data_seek`, `mysqli_free_result`,
`mysqli_more_results`, `mysqli_next_result`, `mysqli_report`, and
`mysqli_init` are currently visible through
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

`mysqli_set_charset($handle, "utf8mb4")` accepts the placeholder object and
returns deterministic `true` for the reached WordPress charset setup shape.
Charset values other than `utf8mb4`, collation state, charset negotiation,
warning/error behavior, and escaping charset effects are not implemented.

`mysqli_query($handle, 'SELECT @@SESSION.sql_mode')` accepts the placeholder
object and that exact SQL mode probe, returning `false` as a deterministic
empty/no-result boundary. This lets WordPress skip SQL mode normalization
without executing SQL or producing a result resource.

`mysqli_query($handle, 'SELECT * FROM wp_posts WHERE 1 = 0')` returns a
placeholder `mysqli_result` object for the first deterministic empty result
lifecycle boundary. `mysqli_num_fields($result)` returns `0`,
`mysqli_num_rows($result)` returns `0`,
`mysqli_fetch_field($result)` and `mysqli_fetch_object($result)` return
`false`, and `mysqli_free_result($result)` returns `null`. For the placeholder
connection, `mysqli_more_results($handle)` and `mysqli_next_result($handle)`
return `false`. This does not execute SQL, store rows, expose real field
metadata, or model real result resources.

`mysqli_query($handle, 'SELECT ID, post_title FROM wp_posts WHERE ID = 1')`
returns a placeholder `mysqli_result` object with deterministic interpreter
state: fields `ID` and `post_title`, plus one row where `ID` is `1` and
`post_title` is `Hello world placeholder`. `mysqli_num_fields()` returns `2`,
`mysqli_num_rows()` returns `1` without advancing the shared row cursor,
`mysqli_fetch_field()` returns `stdClass` objects with a `name` property for
those two fields and then `false`, and `mysqli_fetch_object()` returns one
`stdClass` row object and then `false`. `mysqli_fetch_assoc()` uses the same
row cursor and returns one associative PHP array with keys `ID` and
`post_title` and then `false`. `mysqli_fetch_row()` uses the same row cursor and
returns one numeric PHP array with keys `0` and `1` and then `false`.
`mysqli_fetch_array($result, MYSQLI_ASSOC)`
returns the same associative row shape and then `false`;
`mysqli_fetch_array($result, MYSQLI_NUM)` returns numeric keys `0` and `1`;
`mysqli_fetch_array($result, MYSQLI_BOTH)` and omitted mode/default
`MYSQLI_BOTH` return both numeric and associative keys. The `MYSQLI_ASSOC`,
`MYSQLI_NUM`, and `MYSQLI_BOTH` constants are exposed. Unsupported mode values
remain explicit boundaries. `mysqli_data_seek($result, $offset)` accepts an
integer offset for placeholder results, resets the row cursor when the offset is
in range, and returns `false` for negative or out-of-range offsets. This is a
fixed row-shape and cursor boundary, not SQL execution, database storage,
WordPress content fidelity, broad query/result support, duplicate-column
fidelity, warning/error fidelity, unbuffered result behavior, or real mysqli
metadata.

Other `SELECT` statements are rejected with a specific non-empty-result-set
diagnostic. For example, `SELECT 1` reports that
non-empty `mysqli` result sets are not implemented. This is an explicit
compatibility boundary before general row storage, SQL parsing/execution,
database-backed WordPress queries, and real metadata exist.

`mysqli_select_db($handle, $database)` accepts the placeholder object and a
string or null database name, returning deterministic `true`. It does not
select or validate a real database.

`mysqli_affected_rows($handle)` and `mysqli_insert_id($handle)` accept the
placeholder object and return deterministic `0` for the clean placeholder
connection state. They do not track real mutation queries, insert IDs, errors,
warnings, or transaction state.

`mysqli_ping($handle)` accepts the placeholder object and returns deterministic
`true`. It does not check a real socket, reconnect, update connection state,
emit warnings/errors, or prove host database liveness.

Mutation SQL passed to `mysqli_query()`, currently recognized by leading
`INSERT`, `UPDATE`, `DELETE`, or `REPLACE`, reports an explicit unsupported
diagnostic. It does not change placeholder table state, affected-row metadata,
insert IDs, transactions, errors, or warnings.

Calling `mysqli_connect(...)` is still a stable unsupported runtime boundary:

```text
unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset
```

No real mysqli extension behavior is implemented yet: no host connections, no
real resources or connected objects beyond the placeholder shapes, no real
server metadata, no query execution beyond the documented deterministic
queries, no real database selection beyond deterministic success, no general
non-empty result sets, no real row/field metadata, no charset handling, no
fetch modes beyond the documented placeholder row shapes, no real row-count
state beyond placeholder result counts, no real affected-row or insert-id state
beyond deterministic zero clean state, no real connection liveness checks or
reconnect behavior beyond deterministic ping success, no real
charset/collation negotiation, no errors/warnings, no transactions, no
configuration beyond the current report-mode flag, no PDO bridge, and no
native database lowering.
