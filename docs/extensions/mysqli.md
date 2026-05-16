# mysqli Extension

Status: boundary only.

`mysqli_connect`, `mysqli_real_connect`, `mysqli_get_server_info`,
`mysqli_get_server_version`, `mysqli_get_host_info`, `mysqli_get_client_info`,
`mysqli_get_client_version`, `mysqli_get_proto_info`, `mysqli_thread_id`,
`mysqli_kill`, `mysqli_change_user`, `mysqli_refresh`, `mysqli_get_charset`,
`mysqli_character_set_name`, `mysqli_stat`,
`mysqli_field_count`, `mysqli_close`, `mysqli_options`, `mysqli_set_opt`,
`mysqli_ssl_set`,
`mysqli_connect_errno`, `mysqli_connect_error`,
`mysqli_get_connection_stats`, `mysqli_get_links_stats`,
`mysqli_get_client_stats`, `mysqli_thread_safe`, `mysqli_stmt_init`,
`mysqli_prepare`, `mysqli_stmt_prepare`, `mysqli_stmt_param_count`,
`mysqli_stmt_get_warnings`, `mysqli_stmt_error_list`,
`mysqli_stmt_bind_param`, `mysqli_stmt_bind_result`,
`mysqli_stmt_execute`,
`mysqli_stmt_get_result`, `mysqli_stmt_close`, `mysqli_stmt_errno`,
`mysqli_stmt_error`, `mysqli_stmt_affected_rows`,
`mysqli_stmt_store_result`, `mysqli_stmt_num_rows`, `mysqli_stmt_fetch`,
`mysqli_stmt_result_metadata`, `mysqli_stmt_field_count`,
`mysqli_stmt_free_result`, `mysqli_stmt_data_seek`,
`mysqli_stmt_attr_get`, `mysqli_stmt_attr_set`,
`mysqli_stmt_send_long_data`, `mysqli_stmt_reset`,
`mysqli_stmt_more_results`, `mysqli_stmt_next_result`,
`mysqli_stmt_sqlstate`, `mysqli_stmt_warning_count`,
`mysqli_stmt_insert_id`,
`mysqli_dump_debug_info`,
`mysqli_debug`,
`mysqli_autocommit`,
`mysqli_begin_transaction`, `mysqli_commit`, `mysqli_rollback`,
`mysqli_savepoint`, `mysqli_release_savepoint`,
`mysqli_set_charset`, `mysqli_query`, `mysqli_real_query`,
`mysqli_multi_query`, `mysqli_errno`, `mysqli_error`,
`mysqli_error_list`, `mysqli_sqlstate`, `mysqli_warning_count`, `mysqli_info`,
`mysqli_get_warnings`, `mysqli_affected_rows`,
`mysqli_insert_id`, `mysqli_ping`, `mysqli_select_db`,
`mysqli_real_escape_string`, `mysqli_escape_string`,
`mysqli_fetch_object`, `mysqli_fetch_assoc`, `mysqli_fetch_array`,
`mysqli_fetch_all`, `mysqli_fetch_column`,
`mysqli_fetch_row`, `mysqli_fetch_field`, `mysqli_fetch_fields`, `mysqli_fetch_field_direct`, `mysqli_fetch_lengths`, `mysqli_num_fields`,
`mysqli_num_rows`, `mysqli_data_seek`, `mysqli_field_seek`, `mysqli_field_tell`, `mysqli_free_result`,
`mysqli_more_results`, `mysqli_next_result`, `mysqli_store_result`,
`mysqli_use_result`, `mysqli_reap_async_query`, `mysqli_poll`,
`mysqli_report`, and `mysqli_init` are currently visible
through `function_exists()`, `is_callable()`, dynamic string-valued function
lookup, and native function-table introspection so WordPress' early database
startup paths can move to the next real bootstrap blocker.

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
The optional client-flags argument may be `0` or a combination of the currently
exposed `MYSQLI_CLIENT_*` constants:
`MYSQLI_CLIENT_SSL`, `MYSQLI_CLIENT_COMPRESS`,
`MYSQLI_CLIENT_INTERACTIVE`, `MYSQLI_CLIENT_IGNORE_SPACE`,
`MYSQLI_CLIENT_NO_SCHEMA`, `MYSQLI_CLIENT_FOUND_ROWS`,
`MYSQLI_CLIENT_SSL_VERIFY_SERVER_CERT`,
`MYSQLI_CLIENT_SSL_DONT_VERIFY_SERVER_CERT`, and
`MYSQLI_CLIENT_CAN_HANDLE_EXPIRED_PASSWORDS`. The constants use PHP-matching
integer values and let reached SSL/options setup code pass through the
deterministic connection boundary, but they do not negotiate client
capabilities, configure TLS, verify certificates, mutate connection state, or
affect query/result behavior.

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
allocate server-side threads, or support real reconnect behavior.

`mysqli_kill($handle, $process_id)` accepts the placeholder object and an
integer process id, returning `true` only for the deterministic placeholder
thread id `1` and `false` for other ids. It does not kill a host server
thread, invalidate or reconnect the placeholder connection, emit warnings, or
touch database state.

`mysqli_change_user($handle, $username, $password, $database)` accepts the
placeholder object, string credentials, and a string or null database, returning
deterministic `true`. It does not authenticate, select a real database, reset
server session state, roll back transactions, close temporary tables, unlock
tables, or mutate host connection state.

`mysqli_refresh($handle, $flags)` accepts the placeholder object and a nonzero
integer combination of exposed deprecated `MYSQLI_REFRESH_*` flags, returning
deterministic `true`. The exposed flags are `MYSQLI_REFRESH_GRANT`,
`MYSQLI_REFRESH_LOG`, `MYSQLI_REFRESH_TABLES`, `MYSQLI_REFRESH_HOSTS`,
`MYSQLI_REFRESH_STATUS`, `MYSQLI_REFRESH_THREADS`, `MYSQLI_REFRESH_SLAVE`,
`MYSQLI_REFRESH_REPLICA` as an alias of `MYSQLI_REFRESH_SLAVE`,
`MYSQLI_REFRESH_MASTER`, and `MYSQLI_REFRESH_BACKUP_LOG`. It does not flush
tables, logs, caches, replication state, status counters, host server state,
or connection/session state.

`mysqli_get_charset($handle)` accepts the placeholder object and returns a
deterministic `stdClass`-shaped metadata object for the current utf8mb4
placeholder: `charset = "utf8mb4"`, `collation = "utf8mb4_unicode_520_ci"`,
`dir = ""`, `min_length = 1`, `max_length = 4`, `number = 246`, and
`state = 0`. It does not negotiate or inspect a real connection charset,
reflect client-library/server metadata, track collation changes, or affect
escaping.

`mysqli_character_set_name($handle)` accepts the placeholder object and returns
deterministic `utf8mb4`. It does not inspect, negotiate, or track a real
connection character set.

`mysqli_field_count($handle)` accepts the placeholder object and returns
deterministic clean-state field count `0`. It does not track the most recent
query on the connection, result metadata, or SQL execution state.

`mysqli_close($handle)` accepts the placeholder object and returns
deterministic `true`. It does not close a host connection, invalidate the
placeholder object, release server resources, or affect later placeholder
metadata calls.

`mysqli_options($handle, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, $value)` and its
`mysqli_set_opt()` alias accept bool or int values and return deterministic
`true`. The option constant is exposed with PHP's integer value `201`. This
does not negotiate or apply real client options, change result type conversion,
mutate connection state, or affect later placeholder result rows.
The current placeholder option catalog also exposes and accepts
`MYSQLI_OPT_CONNECT_TIMEOUT`, `MYSQLI_OPT_READ_TIMEOUT`,
`MYSQLI_OPT_NET_CMD_BUFFER_SIZE`, and `MYSQLI_OPT_NET_READ_BUFFER_SIZE` with
integer values; `MYSQLI_INIT_COMMAND` and `MYSQLI_OPT_LOAD_DATA_LOCAL_DIR` with
string values; and `MYSQLI_OPT_LOCAL_INFILE`,
`MYSQLI_OPT_SSL_VERIFY_SERVER_CERT`, and
`MYSQLI_OPT_CAN_HANDLE_EXPIRED_PASSWORDS` with bool or int values. These return
deterministic `true` without storing options, affecting connect/query/result
behavior, validating paths, running init commands, toggling local infile
handling, or changing timeout/network behavior.

`mysqli_ssl_set($handle, $key, $certificate, $ca_certificate, $ca_path,
$cipher_algos)` accepts the placeholder object and string or null SSL option
arguments, returning deterministic `true`. It does not validate files, configure
TLS, mutate connection state, negotiate SSL during `mysqli_real_connect()`, emit
warnings/errors, or inspect host client-library state.

`mysqli_connect_errno()` and `mysqli_connect_error()` return deterministic
clean connect-error state, `0` and `null`. They do not track failed connection
attempts, host extension state, report-mode behavior, or exact PHP warning and
exception behavior.

`mysqli_error_list($handle)` accepts the placeholder object and returns an
empty array for deterministic clean error-list state. It does not track real
warning/error entries, SQLSTATE history, host client-library state, sockets, or
database state.

`mysqli_get_connection_stats($handle)` accepts the placeholder object and
returns an eight-key deterministic statistics array:
`bytes_sent`, `bytes_received`, `packets_sent`, `packets_received`,
`result_set_queries`, `non_result_set_queries`, `connect_success`, and
`active_connections`. Traffic and query counters are zeroed; the connection
counters are deterministic placeholders. This does not model real mysqlnd
statistics, client/server traffic, memory accounting, connection reuse, or host
database state.

`mysqli_get_links_stats()` returns a deterministic zeroed array with `total`,
`active_plinks`, and `cached_plinks`. It does not inspect real persistent
links, sockets, host client-library state, or connection reuse state.

`mysqli_get_client_stats()` returns a small deterministic zeroed array with
`bytes_sent`, `bytes_received`, `packets_sent`, `packets_received`,
`protocol_overhead_in`, `protocol_overhead_out`, `connect_success`, and
`active_connections`. It does not expose PHP's full mysqlnd statistics table,
track real client-library traffic, account for memory, inspect sockets, or
read host database state.

`mysqli_thread_safe()` accepts no arguments and returns deterministic `true`.
It does not inspect host client-library build flags, real thread-safety
configuration, host client-library state, sockets, or database state.

`mysqli_stmt_init($handle)` creates a deterministic placeholder `mysqli_stmt`
object with no prepared query. `mysqli_prepare($handle, $query)` creates a
deterministic placeholder `mysqli_stmt` object and records a simple count of
`?` characters in the query. `mysqli_stmt_close($statement)` removes the
placeholder statement state and returns `true`.

`mysqli_stmt_prepare($statement, $query)` records the query and simple `?`
placeholder count on an existing placeholder statement and returns `true`.
`mysqli_stmt_param_count($statement)` reports that recorded count.
`mysqli_stmt_reset($statement)` clears the recorded query/count and returns
`true`. These helpers do not parse SQL, understand placeholders inside strings
or comments, expose real parameter metadata, bind values, execute statements,
transfer result metadata, track statement diagnostics, touch host database
state, or support native lowering. For the same placeholder statement,
`mysqli_stmt_errno($statement)` returns `0`,
`mysqli_stmt_error($statement)` returns an empty string,
`mysqli_stmt_sqlstate($statement)` returns `00000`,
`mysqli_stmt_warning_count($statement)` returns `0`,
`mysqli_stmt_get_warnings($statement)` returns `false`,
`mysqli_stmt_error_list($statement)` returns an empty array,
`mysqli_stmt_affected_rows($statement)` returns `0`, and
`mysqli_stmt_insert_id($statement)` returns `0`. These are deterministic clean
metadata placeholders only; they do not track failed prepares, executions,
warning-chain objects, error-list entries, affected rows, insert IDs, host
database state, PHP warning/error fidelity, or native lowering.

`mysqli_stmt_execute($statement, $params = null)` executes only the current
unbound placeholder statement shapes. For the seed-post WordPress SELECT, it
records deterministic placeholder result rows; `mysqli_stmt_get_result()` then
returns a placeholder `mysqli_result` containing those rows. Statements with
bound parameters, array parameter execution, mutations, unknown SELECT
metadata, real mysqlnd result transfer, host database state, PHP warning/error
fidelity, and native lowering remain unsupported.

`mysqli_stmt_bind_param($statement, $types, &...$vars)` and
`mysqli_stmt_bind_result($statement, &...$vars)` are visible through callable
metadata but are explicit runtime boundaries. Reached calls report stable
unsupported diagnostics because by-reference parameter/result binding, type
strings, result buffer mutation, fetch integration, and host database execution
are not implemented.

`mysqli_stmt_field_count($statement)` reports deterministic field counts for
the current placeholder statement result metadata shapes, including the
seed-post WordPress SELECT shape. `mysqli_stmt_result_metadata($statement)`
returns a placeholder `mysqli_result` carrying deterministic field metadata
for that seed-post SELECT shape, returns `false` for statements without result
fields, and rejects unknown SELECT metadata with a stable unsupported
diagnostic. `mysqli_stmt_free_result($statement)` validates the active
placeholder statement and returns `null`. This is not prepared binding,
statement execution, statement result rows, mysqlnd result transfer, broad SQL
metadata, host database metadata, PHP warning/error fidelity, or native
lowering.

`mysqli_stmt_store_result($statement)` buffers the deterministic rows recorded
by the current placeholder execution path and returns `true`, or returns
`false` when no placeholder statement result is available.
`mysqli_stmt_num_rows($statement)` reports the buffered placeholder row count,
or `0` before buffering and after `mysqli_stmt_free_result($statement)`.
`mysqli_stmt_fetch($statement)` remains an explicit runtime boundary because
by-reference result binding, output buffer mutation, cursor advancement over
bound buffers, and host database rows are not implemented.

`mysqli_stmt_data_seek($statement, $offset)` records an in-range placeholder
cursor offset for active statements after the current deterministic
`mysqli_stmt_execute()` and `mysqli_stmt_store_result()` path has buffered a
placeholder result. It returns `null` on that bounded path and reports stable
unsupported diagnostics for non-statement handles, unbuffered statements,
non-int offsets, negative offsets, and out-of-range offsets. This is not
`mysqli_stmt_fetch()`, bound-result fetching, by-reference output-buffer
mutation, real mysqlnd cursor behavior, host database state, PHP
warning/error fidelity, or native lowering.

`mysqli_stmt_attr_get($statement, $attribute)` and
`mysqli_stmt_attr_set($statement, $attribute, $value)` expose deterministic
placeholder state for active `mysqli_stmt` objects. The supported attributes
are `MYSQLI_STMT_ATTR_UPDATE_MAX_LENGTH`,
`MYSQLI_STMT_ATTR_CURSOR_TYPE`, and `MYSQLI_STMT_ATTR_PREFETCH_ROWS`; the
cursor-type constants `MYSQLI_CURSOR_TYPE_NO_CURSOR`,
`MYSQLI_CURSOR_TYPE_READ_ONLY`, `MYSQLI_CURSOR_TYPE_FOR_UPDATE`, and
`MYSQLI_CURSOR_TYPE_SCROLLABLE` are exposed with PHP-matching integer values.
Unset attributes return deterministic placeholder defaults. Attribute values
may be int or bool and are stored on the placeholder statement. This is not
real mysqlnd cursor behavior, prefetch behavior, max-length metadata
recalculation, host database state, PHP warning/error fidelity, or native
lowering.

`mysqli_stmt_send_long_data($statement, $param_num, $data)`,
`mysqli_stmt_reset($statement)`, `mysqli_stmt_more_results($statement)`, and
`mysqli_stmt_next_result($statement)` are visible through callable metadata.
`mysqli_stmt_reset($statement)` clears placeholder statement state and returns
`true`. `mysqli_stmt_more_results($statement)` and
`mysqli_stmt_next_result($statement)` return deterministic `false` for active
placeholder statements because no pending statement result queues are modeled.
`mysqli_stmt_send_long_data($statement, $param_num, $data)` remains an
explicit runtime boundary because long-parameter streaming, packet buffering,
statement parameter state, real multi-statement execution, host database
state, PHP warning/error fidelity, and native lowering are not implemented.

`mysqli_stmt_fetch_fields()` and `mysqli_stmt_fetch_field()` are not PHP
mysqli functions and are not exposed by the current function table.

`mysqli_dump_debug_info($handle)` accepts the placeholder object and returns
deterministic `true`. It does not emit MySQL DBUG trace output, inspect host
client-library debug state, inspect sockets, or read host database state.

`mysqli_debug($options)` accepts the current scalar/null string-convertible
options boundary and returns deterministic `true`. It does not parse MySQL
DBUG options, create trace files, mutate host client-library debug state,
inspect sockets, or read host database state.

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

`mysqli_savepoint($handle, $name)` and
`mysqli_release_savepoint($handle, $name)` accept the placeholder object and a
string savepoint name, returning deterministic `true`. They do not create,
release, roll back to, validate, or persist real savepoint state, interact with
host database transactions, emit warnings/errors, or affect later placeholder
transaction calls.

`mysqli_errno($handle)`, `mysqli_error($handle)`,
`mysqli_sqlstate($handle)`, `mysqli_warning_count($handle)`, and
`mysqli_info($handle)` expose deterministic clean placeholder metadata: `0`,
an empty string, `00000`, `0`, and `null`. They do not track real host
database SQLSTATE, warnings, warning counts, statement information strings, or
PHP warning/error behavior.

`mysqli_get_warnings($handle)` accepts the placeholder object and returns
deterministic `false` for clean warning-chain state. It does not expose warning
objects, warning iteration, real SQL warning metadata, or host database state.

`mysqli_set_charset($handle, "utf8mb4")` accepts the placeholder object and
returns deterministic `true` for the reached WordPress charset setup shape.
Charset values other than `utf8mb4`, collation state, charset negotiation,
warning/error behavior, and escaping charset effects are not implemented.

`mysqli_query($handle, 'SELECT @@SESSION.sql_mode')` accepts the placeholder
object and that exact SQL mode probe, returning `false` as a deterministic
empty/no-result boundary. This lets WordPress skip SQL mode normalization
without executing SQL or producing a result resource.

`mysqli_real_query($handle, 'SET NAMES \'utf8mb4\' COLLATE
\'utf8mb4_unicode_520_ci\'')` accepts the placeholder object and that exact
WordPress charset setup statement, returning deterministic `true` without
creating pending result state. `mysqli_real_query()` also accepts the exact
deterministic seed-post and empty-result SQL shapes already supported by
`mysqli_query()`, queues one pending placeholder result on the connection, and
returns `true`. `mysqli_field_count($handle)` reports the pending result field
count until the result is transferred. `mysqli_store_result($handle)` or
`mysqli_use_result($handle)` then consumes that pending state and returns a
placeholder `mysqli_result`; later calls return `false` again. General
result-producing SQL, real buffered/unbuffered transfer, host connection
pending-result queues, multi-result state, mutation state, warning/error
fidelity, and native lowering remain unsupported.

`mysqli_multi_query($handle, 'SET NAMES \'utf8mb4\' COLLATE
\'utf8mb4_unicode_520_ci\'')` accepts the placeholder object and that exact
WordPress charset setup statement, returning deterministic `true`. Real
multi-statement execution, pending result queues,
`mysqli_more_results()`/`mysqli_next_result()` state, result objects, mutation
state, warning/error behavior, and host database state are not implemented.

`mysqli_query($handle, 'SELECT * FROM wp_posts WHERE 1 = 0')` returns a
placeholder `mysqli_result` object for the first deterministic empty result
lifecycle boundary. `mysqli_num_fields($result)` returns `0`,
`mysqli_num_rows($result)` returns `0`,
`mysqli_fetch_field($result)` and `mysqli_fetch_object($result)` return
`false`, and `mysqli_free_result($result)` returns `null`. For the placeholder
connection, `mysqli_more_results($handle)` and `mysqli_next_result($handle)`
return `false`. This does not execute SQL, store rows, expose real field
metadata, or model real result resources.

For the placeholder connection, `mysqli_store_result($handle)` and
`mysqli_use_result($handle)` return deterministic `false` for clean
no-pending-result state, or transfer the current deterministic
`mysqli_real_query()` pending result into a placeholder `mysqli_result`. They
do not transfer real buffered or unbuffered result sets from a host
connection, model result resource modes, support multi-result queues, or expose
real result resources.

`mysqli_reap_async_query($handle)` returns deterministic `false` for the clean
placeholder connection, meaning no async result is pending. `MYSQLI_ASYNC`,
`mysqli_poll()`, async socket readiness, pending async result queues, host
database state, and result object creation are not implemented.

`MYSQLI_ASYNC` is exposed with value `8`, and `mysqli_poll()` is visible for
function/callability metadata. Reached `mysqli_poll()` calls report a stable
unsupported diagnostic because async socket readiness and by-reference
read/error/reject array mutation are not implemented.

`mysqli_query($handle, 'SELECT ID, post_title FROM wp_posts WHERE ID = 1')`
returns a placeholder `mysqli_result` object with deterministic interpreter
state: fields `ID` and `post_title`, plus one row where `ID` is `1` and
`post_title` is `Hello world placeholder`. `mysqli_num_fields()` returns `2`,
`mysqli_num_rows()` returns `1` without advancing the shared row cursor,
`mysqli_fetch_field()` returns deterministic `stdClass` metadata objects for
those two fields and then `false`, `mysqli_fetch_fields()` returns a
zero-indexed array of the current field metadata objects,
`mysqli_fetch_field_direct()` returns one field metadata object by integer
index or `false`, and `mysqli_field_seek()`/`mysqli_field_tell()` mutate and
report the current field cursor. The metadata objects currently expose
`name`, `orgname`, `table`, `orgtable`, `def`, `db`, `catalog`,
`max_length`, `length`, `charsetnr`, `flags`, `type`, and `decimals` for the
deterministic `ID` and `post_title` seed fields. This metadata is fixed
placeholder data, not SQL-derived table/database metadata, protocol
flag/type/collation fidelity, duplicate-column fidelity, or host result
metadata. `mysqli_fetch_lengths()` returns `false` before any row fetch and a
zero-indexed integer array for the most recently fetched row lengths after
`mysqli_fetch_object()`, `mysqli_fetch_assoc()`,
`mysqli_fetch_row()`, `mysqli_fetch_array()`, `mysqli_fetch_all()`, or
`mysqli_fetch_column()`. `mysqli_fetch_object()` returns one `stdClass` row
object and then `false`. `mysqli_fetch_assoc()` uses the same row cursor and
returns one associative PHP array with keys `ID` and `post_title` and then
`false`. `mysqli_fetch_row()` uses the same row cursor and returns one numeric
PHP array with keys `0` and `1` and then `false`.
`mysqli_fetch_array($result, MYSQLI_ASSOC)`
returns the same associative row shape and then `false`;
`mysqli_fetch_array($result, MYSQLI_NUM)` returns numeric keys `0` and `1`;
`mysqli_fetch_array($result, MYSQLI_BOTH)` and omitted mode/default
`MYSQLI_BOTH` return both numeric and associative keys. `mysqli_fetch_all()`
drains all remaining placeholder rows into a zero-indexed outer array; its
default mode is `MYSQLI_NUM`, and the current subset accepts `MYSQLI_ASSOC`,
`MYSQLI_NUM`, and `MYSQLI_BOTH`. `mysqli_fetch_column()` fetches one row from
the shared cursor and returns column `0` by default, a specified integer column
when present, `null` for a missing column, or `false` when no row remains. The
`MYSQLI_ASSOC`, `MYSQLI_NUM`, and `MYSQLI_BOTH` constants are exposed.
Unsupported mode values and non-integer column arguments remain explicit
boundaries. `mysqli_data_seek($result, $offset)` accepts an integer offset for
placeholder results, resets the row cursor when the offset is in range, and
returns `false` for negative or out-of-range offsets. This is a fixed row-shape
and cursor boundary, not SQL execution, database storage, WordPress content
fidelity, broad query/result support, duplicate-column fidelity, warning/error
fidelity, unbuffered result behavior, or real mysqli metadata.

Other `SELECT` statements are rejected with a specific non-empty-result-set
diagnostic. For example, `SELECT 1` reports that
non-empty `mysqli` result sets are not implemented. This is an explicit
compatibility boundary before general row storage, SQL parsing/execution,
database-backed WordPress queries, and real metadata exist.

`mysqli_select_db($handle, $database)` accepts the placeholder object and a
string or null database name, returning deterministic `true`. It does not
select or validate a real database.

`mysqli_real_escape_string($handle, $data)` and its `mysqli_escape_string()`
alias accept the placeholder object and a scalar/null string-convertible value,
returning deterministic MySQL-style escaping for NUL, newline, carriage return,
backslash, single quote, double quote, and Ctrl-Z characters. They do not
inspect a real connection charset, model binary/invalid string behavior, track
connection state, or provide exact MySQL client-library escaping fidelity.

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
state beyond placeholder result counts, no real statement-info state beyond
deterministic null clean state, no real affected-row or insert-id state beyond
deterministic zero clean state, no real connection liveness checks or
reconnect behavior beyond deterministic ping success, no real
connection-level buffered or unbuffered result retrieval beyond clean
no-pending-result placeholders, no real
autocommit or transaction state beyond deterministic autocommit and
begin-transaction/commit/rollback success, no
charset/collation negotiation, no SQLSTATE, warning-count, or warning-chain
tracking, no errors/warnings, no commit/rollback behavior, no
configuration beyond the current report-mode flag, no PDO bridge, and no
native database lowering.
