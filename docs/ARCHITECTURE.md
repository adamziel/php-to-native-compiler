# Architecture

PTN starts with a conservative native backend while preserving the architecture
needed for full PHP compatibility.

Current pipeline:

```text
PHP source
-> lexer tokens with source spans
-> AST
-> PHP-aware IR
-> generated C with boxed-value runtime
-> native executable via system C compiler
```

The generated C backend is the first native-code path, not the final backend.
It lets the project exercise binary production from day one while the runtime
model grows toward full PHP semantics.

Near-term architecture targets:

- Runtime symbol table for variables.
- PHP ordered arrays.
- References and copy-on-write.
- Function and class metadata.
- Diagnostics and exception channels.
- Explicit fallback boundaries for `eval`, variable variables, and runtime
  symbol mutation.

