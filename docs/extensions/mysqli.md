# mysqli Extension

Status: boundary only.

`mysqli_connect`, `mysqli_real_connect`, `mysqli_get_server_info`,
`mysqli_get_server_version`, `mysqli_get_host_info`, `mysqli_get_client_info`,
`mysqli_get_client_version`, `mysqli_get_proto_info`, `mysqli_thread_id`,
`mysqli_get_charset`, `mysqli_stat`, `mysqli_get_connection_stats`, `mysqli_autocommit`,
`mysqli_begin_transaction`, `mysqli_commit`, `mysqli_rollback`,
`mysqli_set_charset`, `mysqli_query`, `mysqli_errno`, `mysqli_error`,
`mysqli_sqlstate`, `mysqli_warning_count`, `mysqli_affected_rows`,
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

`mysqli_get_server_version($handle)` accepts the placeholder object and returns
deterministic integer version `80000`, matching the current fake server string.
It does not query a server, negotiate protocol, inspect server capabilities, or
reflect a real connection.

`mysqli_get_host_info($handle)` accepts the placeholder object and returns
`localhost via TCP/IP (phpc-placeholder)`, deterministic fake connection
metadata for reached host-info probes. It does not inspect a real host,
transport, socket, protocol, or live connection.

`mysqli_get_client_info()` accepts no argument, `null`, or the placeholder
object and returns `mysqlnd 8.0.0-phpc-placeholder`. This is deterministic fake
client-library metadata; it does not inspect the linked client library, model
PHP 8.1 deprecation behavior for passing an argument, or reflect host state.

`mysqli_get_client_version()` accepts no arguments and returns deterministic
integer version `80000`. It does not inspect the linked client library or
reflect extension build configuration.

`mysqli_get_proto_info($handle)` accepts the placeholder object and returns
deterministic protocol version `10`. It does not negotiate or inspect a real
server protocol.

`mysqli_thread_id($handle)` accepts the placeholder object and returns
deterministic thread id `1`. It does not inspect a real server connection,
allocate server-side threads, or support thread killing/reconnect behavior.

`mysqli_get_charset($handle)` accepts the placeholder object and returns a
deterministic `stdClass`-shaped metadata object for the current utf8mb4
placeholder: `charset = "utf8mb4"`, `collation = "utf8mb4_unicode_520_ci"`,
`dir = ""`, `min_length = 1`, `max_length = 4`, `number = 246`, and
`state = 0`. It does not negotiate or inspect a real connection charset,
reflect client-library/server metadata, track collation changes, or affect
escaping.

`mysqli_get_connection_stats($handle)` accepts the placeholder object and
returns an eight-key deterministic statistics array:
`bytes_sent`, `bytes_received`, `packets_sent`, `packets_received`,
`result_set_queries`, `non_result_set_queries`, `connect_success`, and
`active_connections`. Traffic and query counters are zeroed; the connection
counters are deterministic placeholders. This does not model real mysqlnd
statistics, client/server traffic, memory accounting, connection reuse, or host
database state.

`mysqli_stat($handle)` accepts the placeholder object and returns deterministic
zeroed server-status metadata. It does not query real counters, thread/table
state, uptime, live connection status, or server state.

`mysqli_autocommit($handle, bool $mode)` accepts the placeholder object and a
boolean mode, returning deterministic `true`. It does not mutate real
autocommit state, start or end transactions, commit, roll back, emit
warnings/errors, or touch host database state.

`mysqli_begin_transaction($handle, 0, $name)` accepts the placeholder object,
optional flags value `0`, and optional null/string transaction names, returning
deterministic `true`. It does not start real transaction state, mutate
autocommit state, commit, roll back, create savepoints, emit warnings/errors,
or touch host database state.

`mysqli_commit($handle, 0, $name)` and
`mysqli_rollback($handle, 0, $name)` accept the placeholder object, optional
flags value `0`, and optional null/string transaction names, returning
deterministic `true`. They do not commit, roll back, mutate real transaction or
autocommit state, handle savepoints, emit warnings/errors, or touch host
database state.

`mysqli_errno($handle)`, `mysqli_error($handle)`,
`mysqli_sqlstate($handle)`, and `mysqli_warning_count($handle)` expose
deterministic clean placeholder metadata: `0`, an empty string, `00000`, and
`0`. They do not track real host database SQLSTATE, warnings, warning counts,
or PHP warning/error behavior.

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
server, host/transport, client/protocol, connection-statistics, or server-status metadata, no query execution beyond the documented deterministic
queries, no real database selection beyond deterministic success, no general
non-empty result sets, no real row/field metadata, no charset handling, no
fetch modes beyond the documented placeholder row shapes, no real row-count
state beyond placeholder result counts, no real affected-row or insert-id state
beyond deterministic zero clean state, no real connection liveness checks or
reconnect behavior beyond deterministic ping success, no real
autocommit or transaction state beyond deterministic autocommit and
begin-transaction/commit/rollback success, no
charset/collation negotiation, no SQLSTATE or warning-count tracking, no
errors/warnings, no commit/rollback behavior, no
configuration beyond the current report-mode flag, no PDO bridge, and no
native database lowering.
