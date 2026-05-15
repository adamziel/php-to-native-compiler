# mysqli Extension

Status: boundary only.

`mysqli_connect` is currently visible through `function_exists()`,
`is_callable()`, dynamic string-valued function lookup, and native
function-table introspection so WordPress' early missing-extension guard can
move to the next real bootstrap blocker. Calling `mysqli_connect(...)` is still
a stable unsupported runtime boundary:

```text
unsupported call mysqli_connect(): mysqli/database connections are not implemented in the current subset
```

No mysqli extension behavior is implemented yet: no host connections, no
resources or objects, no queries, no result sets, no escaping, no charset
handling, no errors/warnings, no transactions, no configuration, no PDO bridge,
and no native database lowering.
