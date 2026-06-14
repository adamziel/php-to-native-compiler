# PHPT Broad 1k Magic/Object Conversion Frontier: 2026-06-14

Issue: `ptn-3a8d`

This slice maps the broad 1k PHPT rows currently classified as requiring
unsupported magic method dispatch or reflection metadata. The cluster is mostly
`ext/standard/tests/array` rows that combine array helpers with objects,
resources, key conversion, object stringification, comparator callbacks, and
magic methods.

This is a blocker map, not a support claim. PTN has bounded support for public
`__toString()`, `__call`, and `__invoke`, but this broad cluster also needs
helper-wide object/key conversion parity, visibility-sensitive magic hooks,
backtrace metadata, and exact diagnostics. Reopening the whole bucket today
would expose 49 focused failures.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-3a8d-baseline
```

Generated manifest:
`.runtime/ptn-3a8d-baseline-latest/20260614T031909Z/phpt-baseline-1000.txt`

Classification artifact:
`.runtime/phpt-progress/classification-20260614T031909Z.tsv`

Evidence command reported PTN commit: `96a37c7a119d`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 430 | 570 |

Top blocker counts:

| Bucket | Rows |
| --- | ---: |
| PHP attributes | 141 |
| magic method dispatch/reflection metadata | 69 |
| call-site/array unpacking | 34 |
| trait declarations | 25 |
| interface declarations | 23 |
| non-public property visibility metadata | 19 |
| configurable `assert.exception` assertion mode | 17 |
| interface implementation checks | 15 |
| anonymous class syntax | 15 |
| `memory_limit` parsing/enforcement | 15 |

## Focused Frontier

Committed manifest:
`tools/phpt-magic-object-conversion-frontier-manifest.txt`

Selection from `classification-20260614T031909Z.tsv`:

```sh
awk -F'\t' '$3 ~ /magic method dispatch\/reflection metadata/ {print $1}'
```

Classified result:
`.runtime/phpt-progress/run-20260614T032359Z-manifest.log`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 69 | 0 | 69 |

Raw execution with classification disabled:

```sh
PTN_PHPT_CLASSIFY=0 tools/run-bounded-phpt.sh .runtime/ptn-3a8d-magic-method-metadata-rows.txt
```

Result:
`.runtime/phpt-progress/run-20260614T025107Z-manifest.log`

| Selected | Runnable | Passed | Failed | Excluded |
| ---: | ---: | ---: | ---: | ---: |
| 69 | 69 | 20 | 49 | 0 |

## Row Shape

| Group | Rows |
| --- | ---: |
| `ext/standard/tests/array` | 60 |
| Zend asymmetric visibility magic hooks | 4 |
| Zend engine magic/object rows | 4 |
| Zend backtrace in `__toString()` | 1 |

Helper-family split from raw execution:

| Family | Pass | Fail |
| --- | ---: | ---: |
| `array_intersect*` | 4 | 10 |
| `array_diff*` | 2 | 6 |
| `array_merge*` | 2 | 5 |
| `array_udiff*` | 0 | 6 |
| `array_uintersect*` | 0 | 5 |
| Zend asymmetric visibility magic hooks | 0 | 4 |
| Zend engine magic rows | 2 | 2 |
| `array_column*` | 0 | 3 |
| `array_fill*` | 3 | 1 |
| `array_map*` | 1 | 2 |
| `array_combine*` | 0 | 2 |
| `array_reverse*` | 2 | 1 |
| one-row passing helpers | 4 | 0 |
| one-row failing helpers | 0 | 1 |

## Why This Is A Blocker

The cluster is a semantic mix:

- object-to-string conversion must be applied consistently in array helper
  value comparison, key normalization, and callback paths;
- object/resource/array key coercion needs shared helper behavior for
  `array_fill_keys()`, `array_flip()`, `array_combine()`, and key-aware diff
  and intersect helpers;
- user-comparator helpers need parity for object operands, callback argument
  coercion, and diagnostics;
- `__set` and `__unset` rows depend on asymmetric property visibility and magic
  hook dispatch, not only magic method declaration metadata;
- `debug_backtrace()` from inside `__toString()` requires call-frame snapshots
  and argument metadata;
- `define()` and object operation rows need broader object conversion and
  diagnostic ordering semantics.

Twenty rows already pass under raw execution, showing the current
`unsupported-class-metadata` label is a broad safety bucket. The 49 failing rows
show that reopening this classifier wholesale would make broad PHPT noisier.
The next useful implementation slice is not path-based unblocking; it is a
generic object conversion helper used by array key/value conversion, loose
comparison, and callback dispatch, followed by a narrower classifier split for
the passing subset.

## Representative Passing Rows

```text
Zend/tests/bug34260.phpt
Zend/tests/bug34678.phpt
ext/standard/tests/array/array_diff_variation8.phpt
ext/standard/tests/array/array_fill_keys_variation2.phpt
ext/standard/tests/array/array_intersect_variation7.phpt
ext/standard/tests/array/array_merge_variation3.phpt
ext/standard/tests/array/array_reverse_variation5.phpt
```

## Representative Failing Rows

```text
Zend/tests/asymmetric_visibility/__set.phpt
Zend/tests/backtrace/bug39445.phpt
Zend/tests/bug37811.phpt
ext/standard/tests/array/array_column_basic.phpt
ext/standard/tests/array/array_combine_variation4.phpt
ext/standard/tests/array/array_fill_keys_variation1.phpt
ext/standard/tests/array/array_udiff_variation1.phpt
ext/standard/tests/array/array_uintersect_variation1.phpt
```

## Verification

```sh
cargo fmt --check
cargo test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-3a8d-baseline
tools/run-bounded-phpt.sh tools/phpt-magic-object-conversion-frontier-manifest.txt
PTN_PHPT_CLASSIFY=0 tools/run-bounded-phpt.sh .runtime/ptn-3a8d-magic-method-metadata-rows.txt
```
