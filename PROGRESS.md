# Progress

## 2026-06-08

Restarted the project from scratch under `~/ptn-from-scratch`, guided by
`NEW_PROMPT.md`.

Integrated first vertical slice:

- Rust crate `ptn`.
- Lexer and parser for a narrow PHP subset with source spans.
- AST to PHP-aware IR lowering.
- C backend with a boxed-value runtime for `null`, booleans, integers, floats,
  and strings.
- CLI path:
  `ptn compile <input.php> -o <native-binary> [--emit-c]`.
- Native binary exercise test that compiles PHP `echo` into a C-backed native
  executable and runs it.

Unsupported edges are intentionally named rather than hidden: variables, arrays,
functions, classes, includes, references, copy-on-write, resources, exceptions,
and dynamic fallback are not implemented yet.

Next integrated production target:

- Add variables and assignment using a real symbol-table/runtime model that can
  scale to PHP references and copy-on-write.

## 2026-06-09

Recovered the checkpoint worktree onto current `origin/master` at
`ca130c503622ec9a479318d294bfd64d20e496a3` after stale pre-restart bundles
conflicted with the new from-scratch tree.

Integrated the next production slice:

- Lexer support for direct PHP variable tokens such as `$name`.
- Parser/AST support for direct assignment statements and variable reads in
  expressions.
- IR store/load instructions for named variables.
- Generated C runtime symbol table storing boxed `PtnValue` slots by name.
- Native tests proving assignment, reads, and overwrites in compiled binaries.

Still unsupported: arrays, references, copy-on-write, globals/superglobals,
compound assignment, variable variables, undefined-variable warning parity,
functions, classes, resources, exceptions, and dynamic fallback.
