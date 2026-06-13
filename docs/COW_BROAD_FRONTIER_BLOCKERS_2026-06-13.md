# Broad COW/Reference Frontier Blockers: 2026-06-13

Issue: `ptn-flje`

This slice expands the broad COW/reference evidence beyond the focused
`tools/phpt-cow-manifest.txt` set. It does not promote new support claims:
the measured pass subset is below the 25-row threshold, so this is a blocker
map with committed follow-up work.

## Evidence

Source commits:

- PTN: `126ff61ad716`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Candidate construction:

```sh
tools/run-phpt-baseline.sh --tier 5000 --generate-only --out-dir .runtime/ptn-flje/baseline
rg '^- runnable:' docs/COW_BROAD_PHPT_RISK_MAP_2026-06-13.md \
  | sed -E 's/^- runnable: `([^`]+)`.*/\1/' \
  | sort -u > .runtime/ptn-flje/cow-map-runnable.txt
rg -v '^(#|$)' tools/phpt-cow-manifest.txt \
  | sort -u > .runtime/ptn-flje/cow-focused-current.txt
comm -23 .runtime/ptn-flje/cow-map-runnable.txt \
  .runtime/ptn-flje/cow-focused-current.txt \
  > .runtime/ptn-flje/cow-map-runnable-new.txt
```

The resulting committed frontier is
`tools/phpt-cow-broad-frontier-manifest.txt`.

Measured commands:

```sh
tools/run-bounded-phpt.sh --classify-only .runtime/ptn-flje/cow-map-runnable-new.txt
tools/run-bounded-phpt.sh .runtime/ptn-flje/cow-map-runnable-new.txt
```

## Counts

| Measurement | Selected | Runnable | Excluded | Passed | Failed |
| --- | ---: | ---: | ---: | ---: | ---: |
| classify-only | 46 | 42 | 4 | 0 | 0 |
| execution | 46 | 42 | 4 | 11 | 31 |

Refinery revalidation after later classifier work on `ba83e08bf1ff` kept the
same 46 selected rows but classified 31 runnable and 15 excluded rows before
execution. The additional exclusions are source-specific blockers for 11
generator/yield rows; the historical execution row above remains the worker
branch evidence for the original blocker clustering.

Excluded rows:

| Row | Classification |
| --- | --- |
| `Zend/tests/__debugInfo_reference.phpt` | `unsupported-class-metadata`: needs non-public property visibility metadata |
| `Zend/tests/assign_ref_to_overloaded_prop.phpt` | `unsupported-class-metadata`: needs non-public property visibility metadata |
| `ext/standard/tests/array/array_splice_basic.phpt` | `unsupported-internal`: needs `array_splice()` by-reference array mutation plus replacement/reindexing COW separation |
| `ext/standard/tests/array/array_walk/array_walk_recursive.phpt` | `unsupported-internal`: needs recursive by-reference callback traversal and mutation visibility |

The 11 passing rows are useful regression sentinels, but they are not enough
to expand the accepted COW manifest as a standalone pass-count slice:

```text
Zend/tests/array_unshift_COW.phpt
Zend/tests/bw_or_assign_with_ref.phpt
Zend/tests/div_by_zero_compound_refcounted.phpt
Zend/tests/foreach/foreach.phpt
Zend/tests/foreach/foreach_005.phpt
Zend/tests/foreach/foreach_unset_globals.phpt
Zend/tests/foreach/goto_in_foreach.phpt
Zend/tests/indirect_reference_this.phpt
ext/standard/tests/array/array_filter.phpt
ext/standard/tests/array/array_replace_merge_recursive_ref.phpt
ext/standard/tests/array/array_shift_basic.phpt
```

## Blocker Clusters

| Cluster | Rows | Current result | Follow-up |
| --- | ---: | --- | --- |
| Callable prefer-ref diagnostics, inheritance checks, assignment error precedence, and sensitive-parameter reflection | 12 | 8 pass, 3 fail, 1 excluded | `ptn-begn` |
| Closure capture, `Closure::bindTo()`, `Closure::fromCallable()`, and `Closure::__invoke` reference diagnostics | 7 | 0 pass, 7 fail | `ptn-8d2u` |
| Object/property reference targets, overloaded property references, exception reference properties, and foreach-by-ref property writes | 5 | 0 pass, 4 fail, 1 excluded | `ptn-1om3` |
| Plain foreach reference/control rows | 4 | 4 pass, 0 fail | can enter a future pass-count manifest after a 25-row aggregate exists |
| Generator/fiber by-reference yields, returns, cleanup, and foreach iteration | 12 | 0 pass, 12 fail | `ptn-vwyp` |
| Array internal reference parity around replace, splice, recursive walk, filter, and shift | 6 | 3 pass, 1 fail, 2 excluded | `ptn-f0rp` |

## Interpretation

The broad rows are COW/reference sensitive, but most failures are not isolated
copy-on-write payload bugs. They sit behind generic runtime/compiler surfaces:

- Reference diagnostics and callable dispatch now share the fixed-parameter
  prefer-ref/by-ref path across direct calls, `call_user_func()`,
  `call_user_func_array()`, and declared methods. Remaining rows need
  append-form call-argument lvalues, class-name type checks, and
  `SensitiveParameterValue` reflection.
- Closure rows need proper capture validation, closure metadata, callable
  reflection, and `Closure::__invoke` argument names before COW identity is the
  dominant question.
- Object/property rows need generic property reference targets and magic/
  overloaded property diagnostics.
- Generator and fiber rows need a real execution-boundary model for yielding,
  returning, closing, and iterating by reference.
- Array helper rows need remaining mutating-helper reference preservation and
  recursive callback mutation semantics.

Until those follow-ups land, this frontier should be used as a blocker and
regression map rather than an accepted COW passing manifest.
