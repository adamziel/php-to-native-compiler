# PHPT Broad 1k Array Set-Operation Frontier: 2026-06-14

Issue: `ptn-oiin`

This slice used the broad PHPT baseline tooling on `origin/master`, then
focused the `ext/standard/tests/array/array_*diff*` and
`array_*intersect*` rows from the generated 1k tier. The goal was to find a
high-yield generic set-operation implementation slice; the measured frontier
is already mostly green, so this branch records the remaining blocker map
instead of adding row-specific behavior.

The php-src corpus was `/home/claude/php-src-phpt` at revision:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

## Manifest

The broad 1k tier manifest was generated with:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-oiin-before
```

Generated manifest:

```text
.runtime/ptn-oiin-before/20260614T032244Z/phpt-baseline-1000.txt
```

The full classify-only wrapper did not produce a complete 1k summary in this
session; it was terminated after writing an incomplete 892-row classifier file.
The counts below therefore come only from the focused set-operation run.

The focused manifest was derived from the generated broad 1k tier:

```sh
rg '^ext/standard/tests/array/array_.*(diff|intersect)' \
  .runtime/ptn-oiin-before/20260614T032244Z/phpt-baseline-1000.txt \
  > .runtime/ptn-oiin-array-setops-broad.txt
```

## Focused Evidence

Command:

```sh
cargo fmt --check
timeout 900 tools/run-bounded-phpt.sh .runtime/ptn-oiin-array-setops-broad.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T034445Z.tsv
.runtime/phpt-progress/runnable-20260614T034445Z.txt
.runtime/phpt-progress/excluded-20260614T034445Z.tsv
.runtime/phpt-progress/run-20260614T034445Z-manifest.log
```

Result:

| Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 119 | 76 | 43 | 64 | 12 | 0 | 0 |

Excluded rows:

| Category | Rows | Blocker |
| --- | ---: | --- |
| `unsupported-class-metadata` | 33 | Magic method dispatch/reflection metadata in object comparison cases. |
| `unsupported-class-metadata` | 7 | Non-public property visibility metadata in object comparison cases. |
| `unsupported-language` | 3 | Call-site or array unpacking (`...`) in set-operation rows. |

## Runnable Residuals

The 12 runnable failures split into four generic blockers:

| Rows | Failed rows | Blocker |
| ---: | --- | --- |
| 3 | `array_diff_variation9.phpt`, `array_intersect_assoc_variation9.phpt`, `array_intersect_variation9.phpt` | Nested array values in value-based set operations need PHP array-to-string conversion warning parity while preserving comparison output. |
| 2 | `array_intersect_variation3.phpt`, `array_intersect_variation4.phpt` | Heredoc double-quoted escape parity: generated output kept literal `\t`/`\n` bytes where PHP expects tab/newline bytes inside the heredoc value. |
| 2 | `array_udiff_assoc_variation.phpt`, `array_uintersect_assoc_basic2.phpt` | Static includes of helper files that declare comparator functions currently stop with `include files with function or class declarations are unsupported`. |
| 5 | `array_udiff_assoc_variation5.phpt`, `array_udiff_uassoc_variation6.phpt`, `array_udiff_variation5.phpt`, `array_uintersect_assoc_variation5.phpt`, `array_uintersect_uassoc_variation6.phpt` | User comparator arity errors in internal array helpers are fatal in PTN; PHP throws catchable `ArgumentCountError` and continues through the row. |

## Implementation Boundary

No single credible generic change in this slice reaches the 25-row movement
target:

- The largest remaining group is the 43-row class/object/unpacking blocker
  set, which requires broader class metadata, magic dispatch, visibility, and
  unpacking work.
- The runnable failures are real semantics, but each implementation split is
  small in this focused broad tier: catchable comparator arity handling
  accounts for 5 rows, heredoc escape parity for 2 rows, include-declaration
  support for 2 rows, and nested array value warning parity for 3 rows.
- Adding classifier branches for the 12 runnable failures would hide modeled
  runtime semantics, so they remain runnable.

Recommended next implementation order:

1. Shared catchable callback invocation errors for internal comparator helpers.
2. Generic static include support for helper files that declare functions or
   classes before the including script calls them.
3. PHP array-to-string warning parity inside value-based set-operation
   comparison.
4. Heredoc double-quoted escape handling for `\t`/`\n` and related bytes.

## Verification

```sh
cargo fmt --check
timeout 900 tools/run-bounded-phpt.sh .runtime/ptn-oiin-array-setops-broad.txt
```
