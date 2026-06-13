# PHPT COW Foreach/Reference Frontier: 2026-06-13

Issue: `ptn-550s.2`

Focused manifest:

```sh
tools/run-bounded-phpt.sh tools/phpt-cow-foreach-reference-manifest.txt
```

The manifest selects 103 PHPT rows from php-src revision
`8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

```text
Zend/tests/foreach: 58
ext/standard/tests/array/array_walk: 45
```

## Classifier Delta

Initial classify-only evidence on the generated manifest at
`.runtime/phpt-progress/classification-20260613T190245Z.tsv` selected 103
rows, kept 90 runnable, and excluded 13.

This slice added generic classifier blockers for:

- Static local variables.
- Array append reads in foreach iterable expressions.
- Foreach assignment diagnostics for `$this` targets.
- Non-public property visibility metadata.
- `array_walk_recursive()` traversal/mutation semantics.
- By-reference foreach iterator-pointer preservation under positional
  `array_shift()` / `array_unshift()` / `array_splice()` mutation.

Final post-rebase classify-only evidence at
`.runtime/phpt-progress/classification-20260613T193049Z.tsv` selected 103
rows, kept 51 runnable, and excluded 52:

```text
unsupported-class-metadata: 13
unsupported-language: 18
unsupported-ini: 1
unsupported-internal: 20
```

## Final Run

Final bounded evidence:

```text
.runtime/phpt-progress/run-20260613T192143Z-zend-foreach.log
.runtime/phpt-progress/run-20260613T192143Z-array-walk.log
```

Aggregate:

```text
selected: 103
runnable: 51
classified blockers: 52
passed: 31
failed: 20
skipped/warned: 0
```

Bucket split:

| Bucket | Selected | Runnable | Classified | Passed | Failed |
| --- | ---: | ---: | ---: | ---: | ---: |
| Zend foreach | 58 | 35 | 23 | 22 | 13 |
| array_walk | 45 | 16 | 29 | 9 | 7 |

Representative passing rows:

```text
Zend/tests/foreach/foreach_reference.phpt
Zend/tests/foreach/foreach_by_ref_repacking_insert.phpt
Zend/tests/foreach/foreach_temp_array_expr_with_refs.phpt
Zend/tests/foreach/foreach_007.phpt
Zend/tests/foreach/foreach_009.phpt
Zend/tests/foreach/foreach_011.phpt
Zend/tests/foreach/foreach_012.phpt
Zend/tests/foreach/gh11222.phpt
ext/standard/tests/array/array_walk/array_walk_basic1.phpt
ext/standard/tests/array/array_walk/array_walk_variation5.phpt
ext/standard/tests/array/array_walk/bug69068.phpt
ext/standard/tests/array/array_walk/bug69068_2.phpt
```

## Remaining Failure Clusters

### array_walk Callback and Userdata Semantics

Rows:

```text
ext/standard/tests/array/array_walk/array_walk_error2.phpt
ext/standard/tests/array/array_walk/array_walk_variation3.phpt
ext/standard/tests/array/array_walk/array_walk_variation6.phpt
ext/standard/tests/array/array_walk/array_walk_variation8.phpt
ext/standard/tests/array/array_walk/bug12776.phpt
ext/standard/tests/array/array_walk/bug39576.phpt
ext/standard/tests/array/array_walk/bug61730.phpt
```

Follow-up: `ptn-550s.8`.

### foreach Object/Property Targets

Rows:

```text
Zend/tests/foreach/bug34310.phpt
Zend/tests/foreach/bug39017.phpt
Zend/tests/foreach/bug39825.phpt
Zend/tests/foreach/foreach_010.phpt
Zend/tests/foreach/foreach_018.phpt
Zend/tests/foreach/foreach_by_ref_to_property.phpt
```

Follow-up: `ptn-550s.9`.

### foreach List Destructuring Diagnostics

Rows:

```text
Zend/tests/foreach/foreach_list_002.phpt
Zend/tests/foreach/foreach_list_003.phpt
Zend/tests/foreach/foreach_list_004.phpt
Zend/tests/foreach/foreach_list_keyed.phpt
```

Follow-up: `ptn-550s.10`.

### Nested By-Reference Mutation

Rows:

```text
Zend/tests/foreach/bug39036.phpt
Zend/tests/foreach/bug68215.phpt
Zend/tests/foreach/foreach_008.phpt
```

Follow-up: `ptn-550s.11`.

## Verification

```sh
cargo test --test phpt_classifier
cargo test --test cow_native_reducers
cargo test --test foreach_by_ref_cow
tools/run-bounded-phpt.sh tools/phpt-cow-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-cow-foreach-reference-manifest.txt
```
