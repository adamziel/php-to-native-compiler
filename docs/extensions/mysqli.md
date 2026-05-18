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
`mysqli_execute`,
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
`mysqli_execute_query`,
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
deterministic `true` and record placeholder option values per connection.
`MYSQLI_OPT_LOCAL_INFILE` currently affects only the stable
`LOAD DATA LOCAL INFILE` boundary: disabled or unset connections report a
disabled local-infile boundary, while enabled connections report that host file
loading and mutation SQL are still unimplemented. Other accepted options are
recorded without negotiating real client-library options, validating paths,
changing timeout/network behavior, or affecting result rows.
`MYSQLI_INIT_COMMAND` is consulted only by `mysqli_real_connect($handle, ...)`:
exact deterministic no-result init commands such as `SET NAMES utf8mb4` and
the current charset setup shape are accepted without pending result state,
while arbitrary init-command SQL remains an explicit unsupported boundary.

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
unbound placeholder statement shapes and direct-variable bound placeholders
for exact known SQL shapes. For the seed-post WordPress SELECT, it records
deterministic placeholder result rows; `mysqli_stmt_get_result()` then returns
a placeholder `mysqli_result` containing those rows. The exact
`SELECT ID, post_title FROM wp_posts WHERE ID = ?` shape can also execute
after `mysqli_stmt_bind_param($statement, "i", $id)` records a direct
variable and direct `mysqli_stmt_execute()` re-reads its current scalar/null
value from the caller scope. `call_user_func("mysqli_stmt_execute",
$statement)` and positional `call_user_func_array("mysqli_stmt_execute",
array($statement))` use the same refresh path. The exact
`SELECT option_value FROM wp_options WHERE option_name = ?` shape, plus the
same option-value equality shape with `LIMIT 1` and the current backticked
WordPress table/column spelling, reads from the deterministic `wp_options`
state island when a placeholder connection has recorded matching option
state; otherwise it returns an empty deterministic placeholder result.
Positional
`mysqli_stmt_execute($statement, array(...))` params arrays are accepted only
when the array is a PHP list in the current subset, for the exact known SQL
shapes, including through `call_user_func()`. Named/string-keyed params arrays
and sparse integer-keyed params arrays fail with a stable unsupported
diagnostic. Bound `b` parameters can consume recorded
`mysqli_stmt_send_long_data()` chunks for the exact known SQL shapes. Named
params arrays, mutations, unknown SELECT
metadata, real mysqlnd result transfer, real blob packet behavior, host
database state, PHP warning/error fidelity, named-argument callback dispatch,
and native lowering remain unsupported.

`mysqli_execute($statement, $params = null)` is exposed as the procedural
alias for the current `mysqli_stmt_execute()` subset. Direct calls,
string-valued dynamic calls, `call_user_func("mysqli_execute", ...)`, and
positional `call_user_func_array("mysqli_execute", array(...))` use the same
placeholder execution and direct-variable refresh behavior, with
`mysqli_execute()`-specific diagnostics when that alias is the reached API.
This is not a broader statement execution implementation; the same named
params-array, mutation SQL, host database state, PHP warning/error fidelity,
mysqlnd behavior, and native lowering gaps remain.

`mysqli_execute_query($handle, $query, $params = null)` is implemented as a
bounded PHP 8.2+ convenience path over the current placeholder query subset.
It accepts a placeholder `mysqli` object, a string query, and an optional PHP
list params array using the same scalar/null value validation as
`mysqli_stmt_execute($statement, array(...))`. For exact known SELECT shapes it
returns a deterministic placeholder `mysqli_result`; for current deterministic
no-result shapes it returns `true`. It rejects params arrays whose length does
not match the query `?` placeholder count. This is not broad prepared SQL
execution, named params-array support, hidden statement status-copy fidelity,
mutation SQL beyond the documented one-shot `wp_options` delete and upsert
shapes, host database state, PHP warning/error fidelity, mysqlnd behavior, or
native lowering.

For direct `mysqli_query()`, the runtime has one bounded per-placeholder-handle
state island for exact WordPress-shaped option writes and reads:
`INSERT INTO wp_options (option_name, option_value, autoload) VALUES (...)`
records a deterministic `option_id`, the string option value, and autoload
flag, sets `mysqli_affected_rows($handle)` to `1`, and advances deterministic
`mysqli_insert_id($handle)` when the option name is not already recorded. A
duplicate exact plain option insert returns `false`, sets affected rows to
`0`, leaves the existing option id/value/autoload and insert id in place, and
still uses the current clean placeholder diagnostic state. Exact
`INSERT INTO wp_options (option_name, option_value, autoload) VALUES (...)
ON DUPLICATE KEY UPDATE ...` option upserts update existing recorded options
with `mysqli_affected_rows($handle) === 2`, insert missing options with
`mysqli_affected_rows($handle) === 1`, and advance deterministic
`mysqli_insert_id($handle)`. Exact
`REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (...)`
option writes replace existing recorded options with
`mysqli_affected_rows($handle) === 2`, insert missing options with
`mysqli_affected_rows($handle) === 1`, and advance deterministic
`mysqli_insert_id($handle)`. A later exact
`UPDATE wp_options SET option_value = ... WHERE option_name = ...` updates an
existing recorded option with `mysqli_affected_rows($handle) === 1`; missing
option names are successful zero-row updates. A later exact
`UPDATE wp_options SET option_value = ..., autoload = ... WHERE option_name = ...`
updates both the recorded option value and autoload flag with the same
affected-row behavior. A later exact
`UPDATE wp_options SET autoload = ... WHERE option_name = ...` updates only
the recorded autoload flag while preserving the option value with the same
affected-row behavior. A later exact
`DELETE FROM wp_options WHERE option_name = ...` removes an existing recorded
option with `mysqli_affected_rows($handle) === 1`; missing option names are
successful zero-row deletes. A later exact
`DELETE FROM wp_options WHERE option_name IN (...)` removes each distinct
recorded option name in the single-quoted list, reports the number of removed
rows through `mysqli_affected_rows($handle)`, skips missing names, and is also
accepted by `mysqli_execute_query($handle, $query)` with no params. Exact
direct option-name equality reads and equality/`IN` deletes parse
single-quoted option-name literals through the placeholder handle's bounded
SQL mode: after `SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'`, implicit
backslash escapes are disabled for those literals, while the default mode
keeps the current MySQL-style backslash escapes. Exact
prepared `DELETE FROM wp_options WHERE option_name IN (?, ...)` statements,
including the current backticked table/column spelling, are accepted through
`mysqli_stmt_execute()` and `mysqli_execute_query($handle, $query, array(...))`
when every placeholder value is a string option name; duplicate names are
de-duplicated for affected-row accounting. Exact
`DELETE FROM wp_options WHERE option_name LIKE '<pattern>'` removes option
names through the same bounded MySQL-like `LIKE` matcher used by option-row
scans, including `%`, `_`, backslash-escaped wildcard literals, and an exact
trailing single-character `ESCAPE '<char>'` clause for the current plain and
backticked table/column spellings. These direct option-name `LIKE` deletes
honor the placeholder handle's bounded `NO_BACKSLASH_ESCAPES` branch for the
default implicit backslash escape, while explicit `ESCAPE '\\'` keeps using
backslash to escape `%`, `_`, and `\` pattern literals. Prepared
`DELETE FROM wp_options WHERE option_name LIKE ?` shapes, including
backticked table/column spellings and an exact trailing single-character
`ESCAPE '<char>'` clause, remove option names through the same bounded
MySQL-like `LIKE` matcher used by prepared option-row scans, so `%`, `_`, and
escaped wildcard literals such as `\_transient\_%` or `!_transient!_%` are
distinguished. These prepared option-name `LIKE` deletes honor the placeholder
handle's bounded `NO_BACKSLASH_ESCAPES` branch for the default backslash
escape character, while explicit custom `ESCAPE '<char>'` clauses such as
`ESCAPE '!'`, plus explicit `ESCAPE '\\'`, keep using that custom escape
character. This gives the current transient-shaped cleanup probe a
deterministic option-state path for literal/prepared name lists and prepared
LIKE deletes. An exact WordPress-shaped
`DELETE a, b FROM wp_options a, wp_options b ...` transient cleanup shape
also deletes payload rows and matching timeout rows when the supported
payload prefix, timeout prefix, `CONCAT`/`SUBSTRING` timeout expression, and
threshold match. Exact expired-timeout predicates shaped as
`WHERE option_name LIKE ... AND option_value < ...` now use the same bounded
MySQL-like option-name matcher for direct literal, `mysqli_stmt_execute()`,
and `mysqli_execute_query()` paths, including `%`, `_`, backslash-escaped
wildcard literals, and explicit direct literal `ESCAPE '\\'` parity under
`NO_BACKSLASH_ESCAPES`. Prepared
expired-timeout predicates also accept an exact single-character
`ESCAPE '<char>'` clause after `LIKE ?` for the current plain/backticked
table and column spellings, and use the same bounded
`NO_BACKSLASH_ESCAPES` handling as other prepared option-name `LIKE` filters,
including explicit `ESCAPE '\\'` clauses.
This is not broad
`DELETE` SQL, arbitrary multi-table deletes, subquery support, arbitrary
predicates, SQL-mode behavior beyond the bounded direct option-name literal,
prepared option-name `LIKE`, and schema metadata slices, direct literal
pattern projections outside the documented option-name matcher, non-string
option-name params, real index/lock
behavior, or host database execution. A later exact
`SELECT option_value FROM wp_options WHERE option_name = ... LIMIT 1` can
return that value through the existing placeholder `mysqli_result` and fetch
helpers. A later exact
`SELECT autoload FROM wp_options WHERE option_name = ... LIMIT 1` can return
the recorded autoload value through the same placeholder result/fetch path.
The exact
`SELECT option_id FROM wp_options WHERE option_name = ... LIMIT 1` shape can
return the recorded deterministic option id through the same placeholder
result/fetch path.
The exact
`SELECT option_name FROM wp_options WHERE option_name = ... LIMIT 1` shape can
return the recorded option name through the same placeholder result/fetch path,
which is used by the current WordPress-shaped `add_option()` preflight probe.
The exact
`SELECT option_value, autoload FROM wp_options WHERE option_name = ... LIMIT 1`
shape can return the recorded value and autoload columns together through the
same placeholder result/fetch path.
The exact
`SELECT option_name, option_value, autoload FROM wp_options WHERE option_name = ... LIMIT 1`
shape can return the recorded option name, value, and autoload columns
together through the same placeholder result/fetch path.
The exact
`SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = ...`
shape, with or without `LIMIT 1`, can return the deterministic option id plus
recorded name, value, and autoload columns together through the same
placeholder result/fetch path.
Exact option-row reads for
`SELECT option_name, option_value FROM wp_options`,
`SELECT option_name, option_value FROM wp_options WHERE autoload IN ( 'yes',
'on', 'auto-on', 'auto' )`, exact
`SELECT option_name, option_value FROM wp_options WHERE autoload = 'yes'`,
and exact `WHERE option_name IN (...)` shapes return recorded
option-name/option-value rows. The same all-row, autoload-filtered,
autoload-equality, and explicit-name-list shapes are also supported for the
exact `SELECT option_value FROM wp_options ...` projection, returning recorded
option values only, for the
exact `SELECT option_name FROM wp_options ...` projection, returning recorded
option names only, for the exact
`SELECT option_name, autoload FROM wp_options ...` projection, returning
recorded option-name and autoload columns, for the exact
`SELECT option_name, option_value, autoload FROM wp_options ...` projection,
returning recorded option-name, value, and autoload columns, and for the exact
`SELECT option_id, option_name, option_value, autoload FROM wp_options ...`
projection, returning deterministic option-id, name, value, and autoload
columns, and for exact `SELECT * FROM wp_options ...` star projections,
including exact option-name equality with or without `LIMIT 1`, returning the
same deterministic option-id, name, value, and autoload columns.
These row-set projections also accept direct
`WHERE option_name LIKE '<pattern>'` and backtick-quoted
``WHERE `option_name` LIKE '<pattern>'`` filters for deterministic
option-name scans. That direct read path now handles `%` wildcards, `_`
single-character wildcards, backslash-escaped `%`, `_`, and `\` literals, and
a bounded single-character `ESCAPE '<char>'` clause, so transient-shaped scans
such as `_transient_%`, escaped `\_transient\_%`, and explicit custom-escape
patterns can be distinguished. All, autoload-filtered, and LIKE-filtered row
reads use deterministic option-name ordering; explicit `IN (...)` reads
preserve the requested name order and skip missing names.
Missing option names still return an empty placeholder result. The exact
single-quoted literal parser for those direct option shapes accepts the
current MySQL-style backslash escapes used by `mysqli_real_escape_string()`
for quotes, double quotes, backslashes, newlines, and carriage returns, plus
doubled single quotes. This is not broad SQL parsing, SQL-mode-aware escaping,
character-set/collation fidelity, schema/index behavior,
ordering/collation fidelity, autoload mutation beyond the exact insert and
update shapes listed above, SQL `LIKE` wildcard semantics outside those direct
option-name read filters and the bounded schema metadata filters, arbitrary
projection beyond exact option id/name/value/autoload/value-only/name-only/name-value/name-autoload/full-row/full-row-with-id/star-projection shapes,
unique-index enforcement beyond exact plain option-insert duplicate-name
rejection, no-op update affected-row fidelity, real
`REPLACE`/delete-trigger/auto-increment fidelity, DELETE breadth beyond exact
option-name equality, option-name-list, and trailing-percent option-name-prefix
shapes, real
transaction isolation/locking/savepoint behavior, host database execution,
PDO, broad prepared-statement mutation state, warning/error fidelity, or
native lowering. The current transaction and savepoint helpers can snapshot
and restore this exact option state only.

Prepared statement execution over the same state island supports the exact
`SELECT option_value FROM wp_options WHERE option_name = ?` query, plus the
same exact option-value equality query with `LIMIT 1` and the current
backticked WordPress table/column spelling, for string option-name parameters
on the same placeholder handle through
`mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
`mysqli_execute_query($handle, $query, array($name))`; missing names return an
empty placeholder result. The exact
`SELECT option_name, option_value FROM wp_options WHERE option_name = ?` query
also returns a recorded option-name/option-value row for string option-name
parameters on the same handle through the same prepared result paths; missing
names return an empty zero-field placeholder result. The exact
`SELECT option_name, option_value FROM wp_options WHERE option_name IN (?, ...)`,
`SELECT option_value FROM wp_options WHERE option_name IN (?, ...)`,
`SELECT option_value FROM wp_options WHERE autoload IN (?, ...)`,
`SELECT option_name FROM wp_options WHERE option_name IN (?, ...)`,
`SELECT option_name FROM wp_options WHERE autoload IN (?, ...)`,
`SELECT option_name, autoload FROM wp_options WHERE option_name IN (?, ...)`
and
`SELECT option_name, option_value, autoload FROM wp_options WHERE option_name IN (?, ...)`
prepared shapes also return deterministic row sets for string option-name
or autoload-value parameter lists on the same handle through
`mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
`mysqli_execute_query($handle, $query, array(...))`; explicit name-list reads
preserve parameter order, skip missing names, and return empty zero-field
placeholder results when every requested name is missing, while autoload-list
reads sort matching rows by option name and skip unmatched autoload values.
Backticked table/column spellings are accepted for this prepared name-list
slice. The
exact
`SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name IN (?, ...)`
prepared shape also returns deterministic full option rows, including recorded
placeholder option ids, for string option-name parameter lists through the same
prepared result paths; explicit name-list reads preserve parameter order and
skip missing names. Backticked table/column spellings are accepted for this
full-row prepared name-list slice. The
exact
`SELECT option_name FROM wp_options WHERE autoload IN (?, ...)`,
`SELECT option_name, option_value FROM wp_options WHERE autoload IN (?, ...)`,
`SELECT option_name, option_value FROM wp_options WHERE autoload = ?`,
`SELECT option_name, autoload FROM wp_options WHERE autoload IN (?, ...)`,
`SELECT option_name, option_value, autoload FROM wp_options WHERE autoload IN (?, ...)`,
and
`SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE autoload IN (?, ...)`
prepared shapes return deterministic row sets for string autoload parameters
through the same prepared result paths. Matching rows are sorted by option
name, duplicate autoload parameters do not duplicate rows, and missing
autoload values return empty zero-field placeholder results when no rows
match. Backticked table/column spellings are accepted for this prepared
autoload-list/equality slice. Exact prepared
`SELECT option_name, option_value FROM wp_options WHERE option_name LIKE ?`,
`SELECT option_value FROM wp_options WHERE option_name LIKE ?`,
`SELECT option_name FROM wp_options WHERE option_name LIKE ?`,
`SELECT option_name, autoload FROM wp_options WHERE option_name LIKE ?`,
`SELECT option_name, option_value, autoload FROM wp_options WHERE option_name LIKE ?`,
`SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name LIKE ?`,
and `SELECT * FROM wp_options WHERE option_name LIKE ?` shapes also return
deterministic row sets for one string pattern parameter, including backticked
table/column spellings, `%` wildcards, `_` single-character wildcards, and
backslash-escaped `%`, `_`, and `\` literals such as `\_transient\_%`.
Those same prepared read projections also accept a bounded pattern-list form
whose `WHERE` clause is only parenthesized or unparenthesized
`option_name LIKE ? OR option_name LIKE ?` predicates, with one string pattern
parameter per predicate, default backslash escaping, optional backticked
table/column spelling, and the existing trailing `ORDER BY option_name`/`ASC`
suffix. Matching rows are sorted by option name and de-duplicated when multiple
patterns match the same option.
Direct literal option-name `LIKE` scans use the same bounded matcher and now
keep explicit `ESCAPE '\\'` semantics under `NO_BACKSLASH_ESCAPES` for the
documented projection shapes.
These prepared LIKE scans also accept an exact single-character
`ESCAPE '<char>'` clause before the supported trailing `ORDER BY option_name`
or ``ORDER BY `option_name` `` suffix, with optional `ASC`, and keep the
existing deterministic ascending option-name row order. The default backslash
escape character in prepared option-name `LIKE` pattern parameters honors the
placeholder handle's bounded `NO_BACKSLASH_ESCAPES` branch, while explicit
custom single-character `ESCAPE '<char>'` clauses such as `ESCAPE '!'`, plus
explicit `ESCAPE '\\'`, keep using the declared escape character. This
prepared path is still bounded to the documented option-row projections and
does not support prepared pattern-list mutation queries, mixed `AND`/`OR`
predicate groups, per-pattern `ESCAPE` clauses in pattern lists, `DESC`
ordering, arbitrary `ORDER BY` expressions, collation fidelity, or host
database execution. The exact
`SELECT option_name FROM wp_options WHERE option_name = ? LIMIT 1` query
returns a recorded option-name row for string option-name parameters on the
same handle through `mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
`mysqli_execute_query($handle, $query, array($name))`; missing names return an
empty zero-field placeholder result. The exact
`SELECT option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1`
query returns recorded option-value/autoload rows for string option-name
parameters on the same handle through
`mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
`mysqli_execute_query($handle, $query, array($name))`; missing names return an
empty zero-field placeholder result. The exact
`SELECT option_name, option_value, autoload FROM wp_options WHERE option_name = ? LIMIT 1`
query returns recorded option-name/value/autoload rows for string option-name
parameters on the same handle through the same prepared result paths; missing
names return an empty zero-field placeholder result. The exact
`SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE option_name = ?`
query, with or without `LIMIT 1`, returns deterministic
option-id/name/value/autoload rows for string option-name parameters on the
same handle through the same prepared result paths; missing names return an
empty zero-field placeholder result. The exact
`SELECT * FROM wp_options WHERE option_name = ?` query, with or without
`LIMIT 1`, returns the same recorded full option row for string option-name
parameters. The exact
`SELECT option_id FROM wp_options WHERE option_name = ? LIMIT 1` query returns
recorded deterministic option-id rows for string option-name parameters on the
same handle through `mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
`mysqli_execute_query($handle, $query, array($name))`; missing names return an
empty zero-field placeholder result. Prepared no-placeholder row-set reads also
support the exact
`SELECT option_name, option_value FROM wp_options ...`,
`SELECT option_value FROM wp_options ...`,
`SELECT option_name FROM wp_options ...`,
`SELECT option_name, option_value, autoload FROM wp_options ...`, and
`SELECT option_id, option_name, option_value, autoload FROM wp_options ...`,
and `SELECT * FROM wp_options ...`
shapes already accepted by the direct query path, including all rows,
autoload-filtered rows, and literal `option_name IN (...)` lists through
`mysqli_stmt_execute()`/`mysqli_stmt_get_result()` and
`mysqli_execute_query($handle, $query)`. Backticked column/table spellings are
accepted for the current option-name/value row-set slice. The exact
`INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)`
prepared statement records string option-name, option-value, and autoload
parameters on the same handle, updates statement and connection affected-row
metadata to `1`, advances deterministic `mysqli_insert_id($handle)`, and
exposes later exact option-id and option-value reads through the same state
island when the option name is not already recorded. A duplicate exact
prepared plain option insert returns `false`, updates statement and connection
affected-row metadata to `0`, leaves the existing option id/value/autoload and
insert id in place, and still uses the current clean placeholder diagnostic
state. The exact
`INSERT INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)
ON DUPLICATE KEY UPDATE ...` prepared statement records string parameters on
the same handle for the current exact WordPress-style option upsert shapes,
reports affected rows as `2` when updating an existing recorded option and
`1` when inserting a missing option, advances deterministic
`mysqli_insert_id($handle)`, and exposes later exact option-value reads
through the same state island. The same exact upsert shape is also accepted
through one-shot `mysqli_execute_query($handle, $query, array(...))` for
string option-name, option-value, and autoload parameters; it updates
connection affected-row and insert-id metadata but does not create statement
metadata. The exact
`REPLACE INTO wp_options (option_name, option_value, autoload) VALUES (?, ?, ?)`
prepared statement records string parameters on the same handle, reports
affected rows as `2` when replacing an existing recorded option and `1` when
inserting a missing option, advances deterministic `mysqli_insert_id($handle)`,
and exposes later exact option-value reads through the same state island. The exact
`UPDATE wp_options SET option_value = ? WHERE option_name = ?` prepared
statement updates an existing recorded option value for string parameters on
the same handle, updates statement and connection affected-row metadata, and
treats missing option names as successful zero-row updates. The exact
`UPDATE wp_options SET option_value = ?, autoload = ? WHERE option_name = ?`
prepared statement updates both the recorded option value and autoload flag for
string parameters on the same handle with the same affected-row metadata. The
exact
`UPDATE wp_options SET autoload = ? WHERE option_name = ?` prepared statement
updates only the recorded autoload flag for string parameters on the same
handle while preserving the option value, with the same affected-row metadata.
The exact
`DELETE FROM wp_options WHERE option_name = ?` prepared statement removes an
existing recorded option for a string option-name parameter on the same
handle, updates statement and connection affected-row metadata, and treats
missing option names as successful zero-row deletes. The exact
`DELETE FROM wp_options WHERE option_name LIKE ?` prepared statement removes
recorded options whose names match one string MySQL-like pattern parameter,
including `%`, `_`, backslash escapes, and an exact trailing
single-character `ESCAPE '<char>'` clause, and updates statement and
connection affected-row metadata. The exact
prepared WordPress transient pair cleanup shape over `wp_options` aliases `a`
and `b` also removes reached payload rows plus matching timeout rows and
updates statement and connection affected-row metadata. Prepared mutation SQL
without a prior state island remains unsupported. This does not add broad
prepared SQL execution, real unique-index enforcement, no-op update
affected-row fidelity, prepared mutation shapes beyond the exact option value,
value/autoload, autoload-only, insert, replace, upsert, equality delete,
name-list delete, prepared LIKE delete, and transient pair cleanup forms listed
above, arbitrary projections, non-string parameter coercion, result binding
fidelity beyond exact metadata, real auto-increment fidelity, host database
execution, PDO, or native lowering.

`mysqli_stmt_bind_param($statement, $types, &...$vars)` records direct
scalar/null variable snapshots for active statements using `s`, `i`, `d`, or
`b` type markers; direct and callback-dispatched `mysqli_stmt_execute()`
re-read those variables before execution, and recorded long-data chunks
override bound `b` parameter values. This is not true by-reference aliasing,
cross-scope reference cells, named-argument callback dispatch, mutation SQL,
broad SQL execution, real mysqlnd blob behavior, host database state, PHP
warning/error fidelity, or native lowering.

`mysqli_stmt_bind_result($statement, &...$vars)` records direct variable names,
direct variable array-offset targets, direct object-property targets, and
direct object-property array-offset targets for the current known placeholder
statement result shapes. Array-offset keys are evaluated when binding is
registered, so later key-variable changes do not retarget the fetched row.
`mysqli_stmt_fetch($statement)` then copies placeholder row values into those
targets and advances the placeholder cursor. The current bounded path can
fetch directly from the deterministic executed statement result without
`mysqli_stmt_store_result()` first, while `mysqli_stmt_num_rows()` remains
`0` until an explicit store buffers that same placeholder result. This is not
true by-reference aliasing, dynamic object-property target expressions, real
mysqlnd unbuffered network transfer, broad prepared SQL, host database state,
PHP warning/error fidelity, or native lowering.

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
`mysqli_stmt_send_long_data($statement, $param_num, $data)` validates active
statement handles, non-negative in-range parameter indexes, and string chunks,
then records deterministic placeholder chunk state that is cleared by
prepare/reset. Real blob binding, packet buffering, send timing, execution
integration, real multi-statement execution, host database state, PHP
warning/error fidelity, and native lowering are not implemented.

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
boolean mode, returning deterministic `true`; for the current exact
`wp_options` state island, `false` captures a per-handle option-state snapshot
and `true` keeps later option-state changes. It does not mutate real host
autocommit state, execute SQL transactions, emit warnings/errors, or touch
host database state.

`mysqli_begin_transaction($handle, 0, $name)` accepts the placeholder object,
optional flags value `0`, and optional null/string transaction names, returning
deterministic `true`; for the current exact `wp_options` state island it
captures a per-handle option-state snapshot for later rollback. It does not
start real server transaction state, mutate host autocommit state, commit or
roll back host rows, create savepoints, emit warnings/errors, or touch host
database state.

`mysqli_commit($handle, 0, $name)` and
`mysqli_rollback($handle, 0, $name)` accept the placeholder object, optional
flags value `0`, and optional null/string transaction names, returning
deterministic `true`. For the current exact `wp_options` state island, commit
keeps option-state changes and rollback restores the captured per-handle
option-state snapshot. They do not commit, roll back, or isolate real host
database state, mutate real transaction/autocommit state, handle savepoints,
emit warnings/errors, or touch host database state.

`mysqli_savepoint($handle, $name)` and
`mysqli_release_savepoint($handle, $name)` accept the placeholder object and a
string savepoint name, returning deterministic `true`. For the current exact
`wp_options` state island, savepoint records a named per-handle option-state
snapshot, `mysqli_rollback($handle, 0, $name)` restores that named snapshot,
and release removes the named snapshot so later named rollbacks leave current
option state unchanged. They do not create, release, validate, or persist real
host savepoints, implement savepoint nesting diagnostics, roll back host
database state, emit warnings/errors, or touch host transaction state.

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
`mysqli_query($handle, "SET SESSION sql_mode='...'")` also accepts the bounded
WordPress SQL-mode assignment shape when the right-hand side is a single-quoted
empty string or comma-separated uppercase/digit/underscore mode list, returning
deterministic `true` without mutating server session state.

`mysqli_real_query($handle, 'SET NAMES \'utf8mb4\' COLLATE
\'utf8mb4_unicode_520_ci\'')` accepts the placeholder object and that exact
WordPress charset setup statement, returning deterministic `true` without
creating pending result state. `mysqli_real_query()` also accepts the bounded
`SET SESSION sql_mode='...'` assignment shape as a no-result placeholder,
without recording real SQL-mode state. `mysqli_real_query()` also accepts the exact
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
WordPress charset setup statement, returning deterministic `true` without
pending result state. It also accepts the exact deterministic seed-post and
empty-result SQL shapes already supported by `mysqli_real_query()`, queues one
pending placeholder result, and lets `mysqli_store_result()` or
`mysqli_use_result()` consume that result. For semicolon-separated
`mysqli_multi_query()` input, the runtime also accepts a bounded deterministic
multi-result queue when every statement is one of those exact known result
placeholders; `mysqli_more_results()` reports queued future placeholders, and
`mysqli_next_result()` advances after the current pending result is consumed.
Known no-result charset setup statements and the exact
`SELECT @@SESSION.sql_mode` probe and bounded `SET SESSION sql_mode='...'`
assignment shape can also appear before or after those exact result
placeholders; they expose field count `0`, `mysqli_store_result() === false`,
and advance through `mysqli_next_result()`. `mysqli_real_query()` also accepts
the exact SQL-mode probe as a no-result placeholder. Real SQL
execution, broad multi-statement parsing, mutation state, arbitrary no-result
statements, warning/error behavior, host database state, mysqlnd fidelity, and
native lowering are not implemented.

`mysqli_query($handle, 'SELECT * FROM wp_posts WHERE 1 = 0')` returns a
placeholder `mysqli_result` object for the first deterministic empty result
lifecycle boundary. `mysqli_num_fields($result)` returns `0`,
`mysqli_num_rows($result)` returns `0`,
`mysqli_fetch_field($result)` and `mysqli_fetch_object($result)` return
`false`, and `mysqli_free_result($result)` returns `null`. For the placeholder
connection, `mysqli_more_results($handle)` and `mysqli_next_result($handle)`
return `false`. This does not execute SQL, store rows, expose real field
metadata, or model real result resources.

The reached WordPress options-table and metadata read shapes also return
deterministic `mysqli_result` placeholders through `mysqli_query()`:
autoload/all-options reads, option-name priming reads, single-option reads,
and generic empty metadata probes for non-state-island tables. Exact
`SHOW TABLES LIKE 'wp_options'`, `DESCRIBE`/`DESC wp_options`, and
`SHOW [FULL] COLUMNS FROM wp_options` probes return fixed rows for the current
deterministic option-table schema: `option_id`, `option_name`, `option_value`,
and `autoload`, including primary/unique key markers, the `autoload` default,
and placeholder utf8mb4 collation metadata. Exact `SHOW INDEX FROM
wp_options`, `SHOW INDEXES FROM wp_options`, and `SHOW KEYS FROM wp_options`
probes, including backticked table spelling, return fixed MySQL-8-shaped index
rows for the primary `option_id` index and unique `option_name` index. They do
not read a host database, execute arbitrary CREATE/ALTER TABLE, mutate host
schema, inspect real indexes/collations beyond those fixed markers, or model
warning/error fidelity.
For dynamic schema metadata recorded through the bounded `CREATE TABLE` path,
repeating `CREATE TABLE` for an existing recorded table applies a bounded
dbDelta-style diff: declared columns and indexes are upserted by name,
omitted existing columns and indexes are preserved, and table collation is
updated from the new declaration. This does not add full dbDelta SQL
normalization, column rename inference, drop detection from omitted
definitions, index prefix comparison breadth beyond the existing parser,
engine/row-format options, transactional DDL, host database inspection, or
warning/error fidelity.
The same recorded schema also supports direct literal
`SHOW TABLE STATUS WHERE Name IN ('table', ...)` probes,
including ``WHERE `Name` IN (...)`` spelling, accept non-empty
single-quoted identifier-shaped table-name lists through `mysqli_query()`,
returning deterministic table-status rows in table-name order and skipping
missing names. Prepared `SHOW TABLE STATUS WHERE Name = ?` probes, including
``WHERE `Name` = ?`` spelling, return deterministic table-status rows through
`mysqli_execute_query()` and `mysqli_stmt_execute(..., array(...))` when the
single parameter is an identifier-shaped string table name. Missing names
return an empty placeholder result. Prepared
`SHOW TABLE STATUS WHERE Name LIKE ?` probes, including
``WHERE `Name` LIKE ?`` spelling and an exact trailing single-character
`ESCAPE '<char>'` clause, route one string pattern parameter through the same
bounded metadata `LIKE` matcher used by direct schema probes, including the
per-handle `NO_BACKSLASH_ESCAPES` branch and explicit custom escape
characters. Prepared `SHOW TABLE STATUS WHERE Name IN (?, ...)` probes,
including ``WHERE `Name` IN (?, ...)`` spelling, accept a non-empty
placeholder list whose params are all identifier-shaped string table names,
returning deterministic table-status rows in table-name order and skipping
missing names. Bounded table-identifier placeholders are also accepted for
`SHOW [FULL] COLUMNS FROM ?` and `SHOW INDEX`/`SHOW INDEXES`/
`SHOW KEYS FROM ?`, including optional documented `Field`/`Key_name`
equality or `LIKE` filter placeholders as the next parameter. One exact
prepared joined metadata query over
`information_schema.COLUMNS c LEFT JOIN information_schema.STATISTICS s` is
also accepted through `mysqli_execute_query()` and
`mysqli_stmt_execute(..., array(...))` for a single identifier-shaped
`c.TABLE_NAME = ?` parameter. It projects deterministic `Field`, `Type`,
`Null`, `Key`, `Key_name`, `Seq_in_index`, and `Sub_part` rows from the
recorded schema-state island, including null index metadata for columns that
have no recorded index part. This does not add
arbitrary prepared `SHOW TABLE STATUS`
predicates beyond the documented `Name` equality/`LIKE`/`IN` forms,
identifier placeholders outside those schema metadata table positions, joined
metadata queries beyond that exact `COLUMNS`/`STATISTICS` projection, direct
literal joined metadata queries, joined predicates beyond the one table-name
placeholder, exact table
counters/timestamps, host database inspection, or native lowering.

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

`mysqli_connect(...)` accepts zero to six current connection arguments and
returns a placeholder `mysqli` object with deterministic clean connect-error
state. Direct and dynamic string-valued calls use the same constructor, so
WordPress-shaped procedural connection code can pass the placeholder handle
into the existing deterministic query and metadata boundaries. This is not a
host connection, authentication, database selection, init-command execution,
server-state population, or liveness proof.

No real mysqli extension behavior is implemented yet: no host connections, no
real resources beyond the placeholder objects, no real
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
