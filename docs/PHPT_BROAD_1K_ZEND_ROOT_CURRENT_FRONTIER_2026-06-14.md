# PHPT Broad 1k Zend Root Current Frontier: 2026-06-14

Issue: `ptn-vcbw`

This slice refreshes the current broad 1k root-level `Zend/tests/*.phpt`
runnable frontier after the first-class callable and exception-trace work that
landed on `origin/master`. It is a blocker map, not a behavior change. The
remaining failures span array/lvalue semantics, object assignment, control-flow
diagnostics, and legacy engine edge cases; those are shared compiler/runtime
surfaces, not row-local expected-output gaps.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-vcbw-current
```

Source state:

- PTN commit: `74e6e3f21537`
- php-src PHPT corpus: `/home/claude/php-src-phpt`
- corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Artifacts:

```text
.runtime/ptn-vcbw-current/20260614T081649Z/phpt-baseline-1000.txt
.runtime/phpt-progress/classification-20260614T081650Z.tsv
.runtime/phpt-progress/runnable-20260614T081650Z.txt
.runtime/phpt-progress/excluded-20260614T081650Z.tsv
.runtime/phpt-progress/summary-20260614T081650Z.txt
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 425 | 575 |

Top broad classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 16 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

Compared with the pre-rebase broad classifier at
`classification-20260614T072950Z.tsv`, the current branch moves one broad row
from `unsupported-diagnostics-runtime` to broad-runnable:

```text
Zend/tests/ErrorException_getSeverity.phpt
```

The focused execution wrapper below still excludes that row while preparing the
native run, so it remains part of the diagnostics frontier rather than the
root-level execution failures below.

## Focused Zend-Root Evidence

Committed manifest:

```text
tools/phpt-zend-root-current-frontier-manifest.txt
```

Selection:

```sh
awk -F'\t' '$2=="runnable" && $1 ~ /^Zend\/tests\/[^/]+\.phpt$/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T081650Z.tsv \
  > .runtime/ptn-vcbw-zend-root-runnable.txt
```

Focused run:

```sh
tools/run-bounded-phpt.sh .runtime/ptn-vcbw-zend-root-runnable.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T082219Z.tsv
.runtime/phpt-progress/runnable-20260614T082219Z.txt
.runtime/phpt-progress/excluded-20260614T082219Z.tsv
.runtime/phpt-progress/run-20260614T082219Z-manifest.log
.runtime/phpt-progress/summary-20260614T082219Z.txt
```

Result:

| Selected | Runnable | Excluded | Passed | Failed | Skipped | Warned |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 82 | 81 | 1 | 51 | 30 | 0 | 0 |

Green signal in this slice includes arithmetic/addition rows, array addition
and many COW/reference rows, assignment-by-reference cleanup rows, `@` warning
suppression in simple bug rows, and several historical object/reference
regression rows such as `bug31177*`, `bug35163.phpt`, and `bug35239.phpt`.

Focused exclusion:

```text
Zend/tests/ErrorException_getSeverity.phpt
```

## Residual Failure Buckets

| Bucket | Rows | Generic blocker |
| --- | ---: | --- |
| Array/lvalue and ordered-array mutation | 6 | Temporary array append reads, max-next-key overflow, recursive merge next-key handling, iterator cleanup during mutation, and array append/reference parameter behavior need shared ordered-array and lvalue diagnostics. |
| Object/property assignment and conversion | 9 | Object property writes through arrays, null object/dimension assignments, compound object assignment cache semantics, typed-operation diagnostics, `$this` absence, numeric property names, object conversion, and object-reference preservation remain incomplete. |
| Control-flow, scalar formatting, diagnostics, and error handling | 11 | Binary literal formatting, `break` diagnostic parity, typed-argument fatal diagnostics, nested `@` error suppression, comment/open-tag output handling, custom error handler fallback, and string-offset warning suppression require shared parser/runtime diagnostic paths. |
| Dispatch, declaration, callback, and lifecycle edges | 4 | Destructor ordering, dynamic static method calls, method-local function declarations, and invalid object callback validation need generic dispatch/declaration/lifecycle semantics. |

### Array/lvalue and ordered-array mutation

```text
Zend/tests/array_append_reading_error.phpt
Zend/tests/array_literal_next_element_error.phpt
Zend/tests/array_merge_recursive_next_key_overflow.phpt
Zend/tests/array_splice_empty_ht_iter_removal.phpt
Zend/tests/bug34064.phpt
Zend/tests/bug34137.phpt
```

### Object/property assignment and conversion

```text
Zend/tests/assign_array_object_property.phpt
Zend/tests/assign_dim_obj_null_return.phpt
Zend/tests/assign_obj_op_cache_slot.phpt
Zend/tests/assign_op_type_error.phpt
Zend/tests/assign_to_obj_002.phpt
Zend/tests/bug29015.phpt
Zend/tests/bug31098.phpt
Zend/tests/bug31525.phpt
Zend/tests/bug33999.phpt
```

### Control-flow, scalar formatting, diagnostics, and error handling

```text
Zend/tests/binary.phpt
Zend/tests/break_error_001.phpt
Zend/tests/break_error_002.phpt
Zend/tests/break_error_003.phpt
Zend/tests/break_error_004.phpt
Zend/tests/bug33996.phpt
Zend/tests/bug34786.phpt
Zend/tests/bug36513.phpt
Zend/tests/bug37251.phpt
Zend/tests/bug39018.phpt
Zend/tests/bug39018_2.phpt
```

### Dispatch, declaration, callback, and lifecycle edges

```text
Zend/tests/bug20240.phpt
Zend/tests/bug27669.phpt
Zend/tests/bug29104.phpt
Zend/tests/bug31720.phpt
```

## Next Implementation Splits

1. Extend ordered-array/lvalue support for temporary append diagnostics,
   max-next-key overflow, iterator invalidation, and append-by-reference
   behavior before reopening the array mutation rows.
2. Keep object/property assignment separate from array mutation: it needs
   property-name coercion, null receiver diagnostics, object conversion parity,
   and `$this`/method-context handling.
3. Treat `break` diagnostics, binary literal formatting, open-tag/comment
   output, and error-suppression state as control/diagnostic runtime work.
4. Reopen dynamic dispatch/declaration rows only after dynamic static method
   lookup, method-local function declaration timing, invalid object callback
   validation, and destructor ordering are modeled generically.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-vcbw-current
tools/run-bounded-phpt.sh .runtime/ptn-vcbw-zend-root-runnable.txt
cargo fmt --check
```
