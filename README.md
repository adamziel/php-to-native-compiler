# PTN From Scratch

PTN is a fresh PHP-to-native binary compiler project. It is guided by
`NEW_PROMPT.md`: build full PHP compatibility through a generic compiler and
runtime architecture, not through row-shaped test patches.

The current first integrated slice is intentionally small but real:

- PHP source is lexed and parsed into an AST with source spans.
- The AST is lowered into a PHP-aware IR.
- The backend emits C containing a boxed PHP-value runtime.
- The system C compiler produces a native executable.
- Tests compile and execute a generated native binary.

Supported today:

- `<?php` open tag.
- `echo` statements.
- String, integer, float, boolean, and null literals.
- Direct variable assignment and reads for scalar values through the generated
  native runtime symbol table.

Unsupported today:

- Arrays, functions, classes, includes, references, copy-on-write, resources,
  exceptions, compound assignment, undefined-variable diagnostics, variable
  variables, and dynamic fallback. These are architecture targets, not excuses
  for exact-shape hacks.

## Build

```sh
cargo build
```

## Test

```sh
cargo test
```

## Compile a PHP File

```sh
cargo run -- compile examples/hello.php -o /tmp/ptn-hello
/tmp/ptn-hello
```

## Differential Native Output Telemetry

PTN does not yet provide a PHP-compatible `phpc` runner for direct PHPT
execution. For the currently supported subset, compare native output against the
system PHP CLI with:

```sh
tools/diff-native-output.sh --snippet '<?php echo "Hello ", 42, "\n";'
tools/diff-native-output.sh examples/hello.php
```

The command compiles the input through `ptn compile`, runs the produced native
binary, runs the same input with `php`, and compares stdout, stderr, and exit
status. It is telemetry for supported snippets only; it is not a PHPT pass-count
claim.

## Production Workflow

A task is ready only when it is integrated into the branch that will be pushed.
Local experiments and patch files are inventory, not progress.

The production line follows continuous improvement principles:

- Keep small branch-ready changes moving.
- Never stop all development on one integration problem.
- Split oversized work when it blocks flow.
- Make defects visible immediately in `PROGRESS.md`.
- Prefer generic runtime/compiler capabilities over one-off fixes.
- Push integrated progress often so remote history reflects actual movement.
