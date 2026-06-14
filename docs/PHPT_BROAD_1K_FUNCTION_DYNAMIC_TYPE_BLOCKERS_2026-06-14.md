# PHPT Broad 1k Function/Dynamic Type Blockers: 2026-06-14

Issue: `ptn-ingc`

This slice maps a broad 1k parser/control-flow cluster that is not credible as
a one-step implementation patch: function-local `static` storage, nullable and
`never` type metadata, and variable variables. These are generic PHP semantic
surfaces, not row-specific failures.

## Broad 1k Source

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --generate-only --out-dir .runtime/ptn-ingc-baseline
```

Generated manifest:
`.runtime/ptn-ingc-baseline/20260614T051657Z/phpt-baseline-1000.txt`

Corpus revision:
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

## Blocker Counts

The broad 1k source scan found 44 unique rows in this cluster:

| Surface | Rows |
| --- | ---: |
| Function-local `static` variables | 20 |
| Nullable type hints | 16 |
| `never` return type | 2 |
| Variable variables | 8 |
| Unique rows after overlap | 44 |

Focused classify-only evidence after this branch:

```sh
tools/run-bounded-phpt.sh --classify-only .runtime/ptn-ingc-function-dynamic-type-blockers-rows.txt
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 44 | 0 | 44 |

Classifier split:

| Category | Rows |
| --- | ---: |
| `unsupported-language` | 38 |
| `unsupported-class-metadata` | 4 |
| `unsupported-assertion-ini` | 1 |
| `unsupported-opcache-ini` | 1 |

## Classifier Change

The classifier now treats all variable-variable syntax in `--FILE--` as
unsupported dynamic symbol-table behavior, not only `unset($$name)`. This is
the same dynamic fallback zone called out by `NEW_PROMPT.md`: runtime-generated
symbols require an explicit symbol-table/fallback boundary before they should
be counted as runnable native-compiler work.

Focused variable-variable evidence:

```sh
tools/run-bounded-phpt.sh --classify-only .runtime/ptn-ingc-variable_variable-rows.txt
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 8 | 0 | 8 |

Pre-change comparison against `origin/master` for the same 44 rows:

| State | `unsupported-language` | Other excluded | Runnable |
| --- | ---: | ---: | ---: |
| Before | 32 | 8 | 4 |
| After | 38 | 6 | 0 |

Newly classified from runnable:

```text
Zend/tests/arrow_functions/003.phpt
Zend/tests/bug35163_2.phpt
Zend/tests/bug35470.phpt
ext/standard/tests/array/array_combine.phpt
```

## Implementation Boundary

Reopening this cluster requires shared compiler/runtime work:

- function-local `static` storage keyed by function/method identity, with
  initializer timing, references, recursion, closure binding, and shutdown
  behavior;
- nullable parameter and return type metadata plus coercion/diagnostics across
  user functions, closures, methods, default values, and by-reference calls;
- `never` return type control-flow validation for normal returns, throws,
  fatal paths, and callbacks;
- a dynamic symbol-table lookup/mutation boundary for variable variables that
  integrates with globals, locals, references, arrays, diagnostics, and the
  future dynamic fallback architecture.

Until those generic surfaces exist, these rows should remain classified instead
of producing noisy native-run failures.

## Verification

```sh
bash -n tools/phpt-classifier.sh
cargo test --test phpt_classifier
tools/run-bounded-phpt.sh --classify-only .runtime/ptn-ingc-variable_variable-rows.txt
tools/run-bounded-phpt.sh --classify-only .runtime/ptn-ingc-function-dynamic-type-blockers-rows.txt
```
