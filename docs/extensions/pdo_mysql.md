# pdo_mysql Extension

Status: visibility/metadata boundary only.

`extension_loaded("pdo")` and `extension_loaded("pdo_mysql")` return
deterministic `true` through the compiler/runtime compatibility registry, and
`class_exists("PDO")` plus `class_exists("PDOStatement")` report metadata-only
core class entries. `PDO` also exposes a bounded public integer constant
catalog for current metadata checks:

- `PDO::ATTR_ERRMODE`
- `PDO::ERRMODE_SILENT`
- `PDO::ERRMODE_WARNING`
- `PDO::ERRMODE_EXCEPTION`
- `PDO::ATTR_DEFAULT_FETCH_MODE`
- `PDO::FETCH_ASSOC`
- `PDO::FETCH_NUM`
- `PDO::FETCH_BOTH`
- `PDO::MYSQL_ATTR_INIT_COMMAND`

Those constants are available through direct `PDO::CONST`,
`defined("PDO::CONST")`, and `constant("PDO::CONST")` lookup. Unknown PDO
constants remain undefined. This is intended to make PDO-dependent code fail
at an explicit database boundary instead of an undefined-extension,
undefined-class, or common-metadata boundary.

`new PDO(...)` is not implemented. Reached PDO connection attempts report:

```text
unsupported object instantiation for PDO: PDO connections, drivers, statements, and host database state are not implemented in the current subset
```

No full PDO constant catalog, PDO driver, DSN parsing, host connection,
authentication, statement preparation/execution, result fetching,
transactions, attributes, exceptions, error-mode behavior, `PDOStatement`
behavior, `PDOException`, persistent connections, or native database lowering
is implemented.
