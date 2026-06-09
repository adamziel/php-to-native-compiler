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

Current runtime/compiler slices:

- Direct variables lower to generated C `PtnRuntime` symbol-table load/store
  calls.
- Direct variable reads pass through a runtime helper that emits a generic
  undefined-variable warning before yielding `null`.
- Scalar binary `+` and `.` expressions lower to IR value-expression operation
  nodes. The C backend materializes operands into `PtnValue` temporaries in
  source order before calling boxed runtime helpers such as `ptn_add` and
  `ptn_concat`.
- Statement-form `print expr;` lowers to the same boxed output IR instruction
  used by echo, so generated native code routes print output through the
  existing `ptn_echo` helper.

Near-term architecture targets:

- PHP ordered arrays.
- References and copy-on-write.
- Function and class metadata.
- Broader diagnostics and exception channels.
- Full PHP numeric-string conversions, non-numeric string arithmetic
  diagnostics, warnings, and overflow behavior for arithmetic helpers.
- Explicit fallback boundaries for `eval`, variable variables, and runtime
  symbol mutation.
