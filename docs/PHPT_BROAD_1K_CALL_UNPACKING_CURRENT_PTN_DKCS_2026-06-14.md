# PHPT Broad 1k Call Unpacking Current Map: 2026-06-14 ptn-dkcs

Issue: `ptn-dkcs`

This slice refreshes the broad 1k PHPT classifier on the current branch and
rechecks the explicit `unsupported-call-unpacking` frontier. It is a blocker
map, not a runtime implementation claim.

PTN supports variadic parameter binding and first-class callable syntax in
bounded paths, but it still does not model source-level call-site spread
arguments or array-literal spread as a generic parser, IR, and runtime surface.

## Broad 1k Evidence

Command:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-h0qa-current-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h0qa-current-baseline
```

Generated broad manifest:

```text
.runtime/ptn-h0qa-current-baseline/20260614T125733Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/ptn-h0qa-current-progress/summary-20260614T125733Z.txt
.runtime/ptn-h0qa-current-progress/classification-20260614T125733Z.tsv
.runtime/ptn-h0qa-current-progress/runnable-20260614T125733Z.txt
.runtime/ptn-h0qa-current-progress/excluded-20260614T125733Z.tsv
```

State:

```text
PTN: 24318afd2014
php-src PHPT corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Current threshold-sized excluded buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-attribute-syntax-metadata` | 141 |
| `unsupported-object-string-conversion-metadata` | 61 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-trait-declaration` | 25 |

## Focused Call-Unpacking Evidence

Focused manifest:

```text
tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
```

The current broad classifier writes the same 34-row row set to:

```text
.runtime/ptn-h0qa-current-progress/excluded-20260614T125733Z/unsupported-call-unpacking.txt
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-dkcs-call-unpacking-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
```

Focused artifact:

```text
.runtime/ptn-dkcs-call-unpacking-current/classification-20260614T131541Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | Bucket |
| ---: | ---: | ---: | --- |
| 34 | 0 | 34 | `unsupported-call-unpacking` |

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/arg_unpack/` | 13 |
| `Zend/tests/array_unpack/` | 13 |
| `ext/standard/tests/array/` | 6 |
| `Zend/tests/array_unpack_string_keys.phpt` | 1 |
| `Zend/tests/arrow_functions/` | 1 |

## Blocker Boundary

The 34-row frontier is above the broad-slice threshold, but a credible
implementation has to be generic across several compiler/runtime layers:

- parser and AST representation for unpacked call arguments and array literal
  spread elements;
- IR call-argument representation that preserves PHP evaluation order across
  ordinary, named, and unpacked operands;
- call dispatch expansion with positional/named argument merge rules,
  duplicate-key diagnostics, by-reference binding, and internal function
  argument parsing;
- array literal spread semantics for integer and string keys, invalid operand
  diagnostics, reference/COW behavior, and future traversable operands;
- diagnostics for spread positions that remain unsupported, including
  destructuring contexts.

The parser currently lowers calls to `Vec<Expr>` plus optional argument names,
and the IR/backend preserve that flat shape through `InternalCall`,
`DynamicCall`, `MethodCall`, and `NewObject`. Treating `...` as just another
expression would lose PHP's merge, binding, and diagnostic semantics, so this
slice keeps the frontier classified instead of adding partial call-specific
behavior.

## Verification

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-h0qa-current-progress \
  tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h0qa-current-baseline
PHPT_PROGRESS_DIR=.runtime/ptn-dkcs-call-unpacking-current \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
cargo fmt --check
cargo test --test phpt_classifier
```

Current integration results:

- Broad 1k classify-only: 1,000 selected, 424 runnable, 576 excluded.
- Focused call-unpacking replay: 34 selected, 0 runnable, 34 excluded.
- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: passed, 36 tests.
