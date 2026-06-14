# PHPT Broad 1k Diff/Intersect Frontier: 2026-06-14

Issue: `ptn-c8z6`

This slice refreshes the broad 1k `array_diff*`, `array_intersect*`,
`array_udiff*`, and `array_uintersect*` frontier after `ptn-igxz` landed
generic set-operation diagnostics and `ptn-4fd3` made additional plain
heredoc/nowdoc rows runnable. The remaining failures are split across
string-comparison bytes/warnings, include-time declarations, and callback
exception behavior, so this commit records a current blocker map rather than a
partial implementation.

## Broad Baseline

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-c8z6-after-heredoc
```

Generated manifest:
`.runtime/ptn-c8z6-after-heredoc/20260614T023845Z/phpt-baseline-1000.txt`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Current-branch classifier output:

| Bucket | Rows |
| --- | ---: |
| Selected | 1,000 |
| Runnable | 430 |
| Excluded | 570 |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |
| Other classifier buckets | 25 |

Artifacts:

- `.runtime/phpt-progress/classification-20260614T023845Z.tsv`
- `.runtime/phpt-progress/runnable-20260614T023845Z.txt`
- `.runtime/phpt-progress/summary-20260614T023845Z.txt`

## Focused Manifest

The focused manifest selected all currently runnable broad rows in the
diff/intersect and user-comparator families:

```sh
awk -F '\t' '$2=="runnable" && $1 ~ /^ext\/standard\/tests\/array\/array_(u?diff|u?intersect)/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T023845Z.tsv \
  > .runtime/ptn-c8z6-diff-intersect-heredoc-manifest.txt
tools/run-bounded-phpt.sh .runtime/ptn-c8z6-diff-intersect-heredoc-manifest.txt
```

Focused run log:
`.runtime/phpt-progress/run-20260614T024433Z-manifest.log`

| Focused set | Selected | Runnable | Passed | Failed |
| --- | ---: | ---: | ---: | ---: |
| Broad diff/intersect + user comparators | 76 | 76 | 64 | 12 |

Family split:

| Family | Selected | Passed | Failed |
| --- | ---: | ---: | ---: |
| `array_diff*` | 39 | 38 | 1 |
| `array_intersect*` | 30 | 26 | 4 |
| `array_udiff*` | 4 | 0 | 4 |
| `array_uintersect*` | 3 | 0 | 3 |

Compared with the 68-row post-`ptn-igxz` focused set, the latest classifier
adds eight `array_intersect*` variation rows. Six of those pass; the two new
failures are multiline string byte-parity cases in
`array_intersect_variation3.phpt` and `array_intersect_variation4.phpt`.

## Remaining Blockers

### Value Stringification, Bytes, and Warnings

5 rows still fail in plain diff/intersect value comparison:

```text
ext/standard/tests/array/array_diff_variation9.phpt
ext/standard/tests/array/array_intersect_assoc_variation9.phpt
ext/standard/tests/array/array_intersect_variation3.phpt
ext/standard/tests/array/array_intersect_variation4.phpt
ext/standard/tests/array/array_intersect_variation9.phpt
```

The variation 9 rows compare nested arrays and expect repeated
`Array to string conversion` warnings while completing the operation. The newly
runnable variation 3/4 rows expose multiline string byte parity: PTN preserves
literal `\t` and `\n` text where PHP has tab/newline bytes. The generic fix
belongs in shared value-to-comparison-string and string-literal byte handling,
not in individual array helper wrappers.

### Include Files with Comparator Declarations

2 rows use `include('compare_function.inc')` to define the comparator function
used by the set operation:

```text
ext/standard/tests/array/array_udiff_assoc_variation.phpt
ext/standard/tests/array/array_uintersect_assoc_basic2.phpt
```

PTN currently rejects include files that contain function or class declarations
on this path. Reopening these rows should come from generic include-time
declaration integration so included functions become visible to later callback
resolution.

### Comparator Exception and Return Semantics

5 rows reach user comparators but differ when comparator arity or return values
are wrong:

```text
ext/standard/tests/array/array_udiff_assoc_variation5.phpt
ext/standard/tests/array/array_udiff_uassoc_variation6.phpt
ext/standard/tests/array/array_udiff_variation5.phpt
ext/standard/tests/array/array_uintersect_assoc_variation5.phpt
ext/standard/tests/array/array_uintersect_uassoc_variation6.phpt
```

The current failure shape is an uncaught fatal "expects at least 3 arguments"
diagnostic from the callback boundary. PHP catches a `Throwable` for too-many
parameter callbacks and continues to later comparator cases, including
incorrect return values and too-few parameter behavior.

## Next Implementation Split

The most credible generic implementation order is:

1. Centralize set-operation value-to-comparison-string conversion with
   array/object warning behavior and byte-exact string input.
2. Let includes contribute declared functions/classes to the caller-visible
   symbol tables.
3. Route user-comparator arity failures through catchable callback invocation
   so `try`/`catch` around array helpers can observe them.
