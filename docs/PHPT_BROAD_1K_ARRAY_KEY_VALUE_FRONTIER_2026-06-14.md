# PHPT Broad 1k Array Key/Value Frontier: 2026-06-14

Issue: `ptn-odca`

This slice starts from the broad 1k PHPT manifest and narrows the standard
array key/value helper frontier. It records a blocker map instead of claiming a
new implementation move: the remaining failures split across range/string
generation, binary string escaping, resource/directory handles, object
diagnostics, and float-key conversion notices.

## Broad Source

The focused manifest was derived from the generated broad 1k manifest:

```text
.runtime/ptn-odca-baseline-before/20260614T025528Z/phpt-baseline-1000.txt
```

Corpus revision:

```text
8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Selection:

```sh
awk '$0 !~ /^#/ && NF && $0 ~ /^ext\/standard\/tests\/array\/(array_(key_exists|key_first|key_last|keys|search|flip|count_values|combine)|in_array)/ {print}' \
  .runtime/ptn-odca-baseline-before/20260614T025528Z/phpt-baseline-1000.txt \
  > tools/phpt-array-key-value-frontier-manifest.txt
```

## Focused Evidence

Command:

```sh
tools/run-bounded-phpt.sh tools/phpt-array-key-value-frontier-manifest.txt
```

Artifacts:

```text
.runtime/phpt-progress/summary-20260614T031706Z.txt
.runtime/phpt-progress/classification-20260614T031706Z.tsv
.runtime/phpt-progress/run-20260614T031706Z-manifest.log
```

Result:

| Selected | Runnable | Classified | Passed | Failed |
| ---: | ---: | ---: | ---: | ---: |
| 42 | 37 | 5 | 28 | 9 |

Classified rows:

| Rows | Bucket | Representative rows |
| ---: | --- | --- |
| 4 | `unsupported-class-metadata` | `array_combine_variation4.phpt`, `array_combine_variation5.phpt`, `array_flip_variation4.phpt`, `array_key_exists_variation1.phpt` |
| 1 | `harness-cleanup` | `array_count_values_variation.phpt` |

## Passing Shape

The 28 passing rows cover the current generic floor for this cluster:

- `array_combine_basic`, empty/error, binary-safe combine, and common
  integer/string key conversion.
- `array_count_values` basic counting.
- `array_flip` basic keys, repeated values, and common valid-value cases.
- `array_key_exists` ordinary keys, references, multidimensional arrays,
  equality probes, pointer stability, and mixed key types.
- `array_key_first`, `array_key_last`, and most `array_keys` filter paths.
- `array_search` ordinary loose/strict searches and scalar haystack variants.

## Remaining Blockers

The nine runnable failures do not share one implementation primitive:

| Rows | Blocker | Representative rows |
| ---: | --- | --- |
| 1 | `range()` string endpoints and dynamic-variable reads. `array_combine.phpt` builds `$letters = range('a', 'p')` and then reads `$$letter`; PTN currently models integer-convertible `range()` values, so the row reaches `$0` instead of `$a`...`$p`. |
| 2 | Binary string and heredoc escape parity for array keys/values. `array_combine_variation3.phpt` and `array_flip_variation3.phpt` expose tab/newline rendering and embedded-NUL key handling gaps. |
| 2 | Warning formatting for skipped non-string/non-int values. `array_count_values2.phpt` and `array_flip.phpt` differ on PHPT warning/newline shape around skipped values. |
| 1 | Float-to-int array-key deprecation notices. `array_key_exists_variation3.phpt` finds keys correctly but lacks the precision-loss deprecation messages for float key probes. |
| 2 | Directory resource internals. `array_keys_variation_005.phpt` and `array_search_variation4.phpt` require `opendir()`/`closedir()` directory resources in addition to stream resources. |
| 1 | Object class-name diagnostics for internal array arguments. `array_search_variation3.phpt` reports `object given`; PHP expects the concrete class name. |

## Follow-Up Shape

The next credible implementation split should not patch these rows directly:

1. Extend `range()` to PHP string endpoint semantics, then re-run
   `array_combine.phpt` and the broader scalar/string range rows.
2. Reuse the binary-safe string literal/dump path for embedded-NUL array keys
   and heredoc escape handling before expanding combine/flip evidence.
3. Add directory stream resources (`opendir()`, `closedir()`, `readdir()` shape)
   through the existing resource table, then re-run filesystem/path and
   key/search resource rows together.
4. Centralize internal TypeError operand naming so object operands use concrete
   class names consistently.
5. Route float array-key probes through the shared scalar conversion warning
   path so precision-loss deprecations are not helper-specific.

Until one of those primitives lands, this 42-row focused manifest is a more
useful regression target than the entire standard-array bucket.

## Verification

```sh
cargo fmt --check
tools/run-bounded-phpt.sh tools/phpt-array-key-value-frontier-manifest.txt
```
