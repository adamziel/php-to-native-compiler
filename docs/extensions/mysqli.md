# mysqli Extension

Status: boundary only.

`mysqli_connect`, `mysqli_report`, and `mysqli_init` are currently visible
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

Calling `mysqli_connect(...)` is still a stable unsupported runtime boundary:

```text
unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset
```

No real mysqli extension behavior is implemented yet: no host connections, no
real resources or connected objects, no queries, no result sets, no escaping,
no charset handling, no errors/warnings, no transactions, no configuration
beyond the current report-mode flag, no PDO bridge, and no native database
lowering.
