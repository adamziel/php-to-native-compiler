# PHPT Broad 1k Unpacking Classifier Split: 2026-06-14 ptn-ei36

Issue: `ptn-ei36`

This slice refines the broad PHPT unpacking blocker. The previous
`unsupported-call-unpacking` bucket grouped two different source-level
constructs: call-site spread arguments and array/destructuring unpacking. PTN
still does not support either runtime surface, but they need different parser,
IR, call-dispatch, and array-lowering work. Keeping them in separate telemetry
buckets gives future implementation slices a cleaner target.

This is classifier behavior and blocker-map work, not a runtime support claim.

## Classifier Change

The PHPT classifier now inspects sanitized `--FILE--` code around `...` and
uses the immediate delimiter context:

| Context | Bucket |
| --- | --- |
| call/new/function argument list | `unsupported-call-unpacking` |
| short array, `array(...)`, `list(...)`, or destructuring context | `unsupported-array-unpacking` |

Variadic parameter declarations and first-class callable syntax remain
runnable/classified by their existing rules.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ei36-current-baseline
```

Generated broad manifest:

```text
.runtime/ptn-ei36-current-baseline/20260614T131920Z/phpt-baseline-1000.txt
```

Artifacts:

```text
.runtime/ptn-ei36-current-progress/summary-20260614T131921Z.txt
.runtime/ptn-ei36-current-progress/classification-20260614T131921Z.tsv
.runtime/ptn-ei36-current-progress/runnable-20260614T131921Z.txt
.runtime/ptn-ei36-current-progress/excluded-20260614T131921Z.tsv
```

State:

```text
PTN: 3953cf9bd901 plus ptn-ei36 classifier replay
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Broad result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Unpacking split:

| Bucket | Rows | Focused manifest |
| --- | ---: | --- |
| `unsupported-call-unpacking` | 20 | `tools/phpt-call-unpacking-current-ptn-ei36-manifest.txt` |
| `unsupported-array-unpacking` | 14 | `tools/phpt-array-unpacking-current-ptn-ei36-manifest.txt` |
| Total unpacking rows | 34 | |

The broad selected/runnable/excluded totals stay unchanged from the prior
34-row aggregate bucket; the movement is the telemetry split.

## Focused Evidence

Command:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-ei36-current-call \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-current-ptn-ei36-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-ei36-current-array \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-array-unpacking-current-ptn-ei36-manifest.txt
```

Artifacts:

```text
.runtime/ptn-ei36-current-aggregate/classification-20260614T131920Z.tsv
.runtime/ptn-ei36-current-call/classification-20260614T131920Z.tsv
.runtime/ptn-ei36-current-array/classification-20260614T131920Z.tsv
```

Focused result:

| Selected | Runnable | Excluded | `unsupported-call-unpacking` | `unsupported-array-unpacking` |
| ---: | ---: | ---: | ---: | ---: |
| 34 | 0 | 34 | 20 | 14 |

## Row Shape

`unsupported-call-unpacking`:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/arg_unpack/` | 13 |
| `Zend/tests/arrow_functions/` | 1 |
| `ext/standard/tests/array/` | 6 |

`unsupported-array-unpacking`:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/array_unpack/` | 13 |
| `Zend/tests/array_unpack_string_keys.phpt` | 1 |

## Implementation Boundary

Call-site unpacking needs argument-vector expansion in evaluation order,
named/positional merge rules, by-reference binding, internal-call arity and
type diagnostics, and object/traversable iteration at the spread boundary.

Array and destructuring unpacking needs array-element spread lowering, integer
and string key overwrite/reindexing rules, invalid operand diagnostics,
reference/COW behavior, and diagnostics for unsupported spread positions such
as destructuring contexts.

These are related but separable compiler/runtime layers, so the classifier now
keeps the 34 broad rows split into their two implementation targets.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-frontier-ptn-wd68-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-ei36-current-call \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-call-unpacking-current-ptn-ei36-manifest.txt
PHPT_PROGRESS_DIR=.runtime/ptn-ei36-current-array \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-array-unpacking-current-ptn-ei36-manifest.txt
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-ei36-current-baseline
```

Results:

- `cargo fmt --check`: passed.
- `cargo test --test phpt_classifier`: 36 passed.
- Focused unpacking classifier: 34 selected, 0 runnable, 34 excluded; 20
  call-site unpacking rows and 14 array-unpacking rows.
- Focused call-site manifest: 20 selected, 0 runnable, 20 excluded as
  `unsupported-call-unpacking`.
- Focused array manifest: 14 selected, 0 runnable, 14 excluded as
  `unsupported-array-unpacking`.
- Broad 1k classifier: 1,000 selected, 424 runnable, 576 excluded; 20
  `unsupported-call-unpacking` rows and 14 `unsupported-array-unpacking` rows.
