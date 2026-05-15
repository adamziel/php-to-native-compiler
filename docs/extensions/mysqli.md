# mysqli Extension

Status: boundary only.

`mysqli_connect` and `mysqli_report` are currently visible through
`function_exists()`, `is_callable()`, dynamic string-valued function lookup,
and native function-table introspection so WordPress' early database startup
paths can move to the next real bootstrap blocker.

`mysqli_report($mode)` accepts the current WordPress startup mode
`MYSQLI_REPORT_OFF` and the common PHP 8 strict mode combination
`MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT`, records the mode, and returns
`true`. Report-mode warning/error routing is not implemented.

Calling `mysqli_connect(...)` is still a stable unsupported runtime boundary:

```text
unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset
```

No real mysqli extension behavior is implemented yet: no host connections, no
`mysqli_init()`, no resources or objects, no queries, no result sets, no
escaping, no charset handling, no errors/warnings, no transactions, no
configuration beyond the current report-mode flag, no PDO bridge, and no native
database lowering.
