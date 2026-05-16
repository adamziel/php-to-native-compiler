# pdo_mysql Extension

Status: visibility boundary only.

`extension_loaded("pdo")` and `extension_loaded("pdo_mysql")` return
deterministic `true` through the compiler/runtime compatibility registry, and
`class_exists("PDO")` plus `class_exists("PDOStatement")` report metadata-only
core class entries. This is intended to make PDO-dependent code fail at an
explicit database boundary instead of an undefined-extension or undefined-class
boundary.

`new PDO(...)` is not implemented. Reached PDO connection attempts report:

```text
unsupported object instantiation for PDO: PDO connections, drivers, statements, and host database state are not implemented in the current subset
```

No PDO driver, DSN parsing, host connection, authentication, statement
preparation/execution, result fetching, transactions, attributes, exceptions,
error modes, `PDOStatement` behavior, `PDOException`, persistent connections,
or native database lowering is implemented.
