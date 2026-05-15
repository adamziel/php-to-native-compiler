# mysqli Extension

Status: boundary only.

`mysqli_connect`, `mysqli_real_connect`, `mysqli_get_server_info`,
`mysqli_query`, `mysqli_select_db`, `mysqli_report`, and `mysqli_init` are currently visible through
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

`mysqli_select_db($handle, $database)` accepts the placeholder object and a
string or null database name, returning deterministic `true`. It does not
select or validate a real database.

Calling `mysqli_connect(...)` is still a stable unsupported runtime boundary:

```text
unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset
```

No real mysqli extension behavior is implemented yet: no host connections, no
real resources or connected objects beyond the placeholder shape, no real
server metadata, no query execution beyond the current false SQL mode probe, no
real database selection beyond deterministic success, no result sets, no
escaping, no charset handling, no errors/warnings, no transactions, no
configuration beyond the current report-mode flag, no PDO bridge, and no
native database lowering.
