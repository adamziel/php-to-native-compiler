# Broad PHPT 1k Unsupported-Language Frontier: 2026-06-14

Issue: `ptn-a0r0`

This slice refreshes the broad 1k classifier on `origin/master` and maps the
current `unsupported-language` bucket. It is a blocker map, not an
implementation claim: the selected rows require parser, AST/IR, metadata,
symbol-table, and call-lowering surfaces that should move as generic PHP
semantics rather than row-specific expected-output fixes.

## Evidence

Source state:

- PTN: `2bc1951166e8`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-a0r0-final
```

Generated broad manifest:

```text
.runtime/ptn-a0r0-final/20260614T063651Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T063651Z.tsv
.runtime/phpt-progress/runnable-20260614T063651Z.txt
.runtime/phpt-progress/excluded-20260614T063651Z.tsv
```

Broad 1k classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Largest classifier buckets in this run:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |

## Unsupported-Language Split

Reason split from `classification-20260614T063651Z.tsv`:

| Generic blocker | Rows |
| --- | ---: |
| PHP attribute syntax (`#[...]`) plus reflection/declaration metadata | 141 |
| Call-site or array unpacking (`...`) | 34 |
| Trait declarations | 25 |
| Interface declarations | 23 |
| Interface implementation checks | 15 |
| Anonymous class syntax (`new class`) | 15 |
| Nullable type-hint metadata and coercion (`?T`) | 14 |
| Static local variables | 11 |
| Variable variables and runtime symbol-table lookup/mutation | 8 |
| Named-argument binding for modeled array internals | 1 |
| Generator/yield lowering | 1 |
| Total | 288 |

Path split:

| Path group | Rows |
| --- | ---: |
| `Zend/tests/attributes/` | 187 |
| `Zend/tests/anon/` | 22 |
| `Zend/tests/arg_unpack/` | 14 |
| `Zend/tests/array_unpack/` | 13 |
| `ext/standard/tests/array/` | 11 |
| `Zend/tests/ArrayAccess/` | 10 |
| `Zend/tests/backtrace/` | 5 |
| `Zend/tests/arrow_functions/` | 5 |
| `Zend/tests/autoload/` | 2 |
| `Zend/tests/assert/` | 2 |
| Single root Zend/basic rows | 17 |

## Cross-Map

The 288 rows overlap several focused blocker maps that remain valid, but this
run is the current top-level language bucket for the generated broad 1k
manifest.

| Surface | Rows | Existing adjacent map |
| --- | ---: | --- |
| PHP attributes and attribute-adjacent metadata | 141 direct attribute-syntax rows; 187 rows under `Zend/tests/attributes/` | `PHPT_BROAD_1K_ATTRIBUTE_METADATA_FRONTIER_PTN_LZEF_2026-06-14.md` |
| Call-site/array unpacking | 34 | `PHPT_BROAD_1K_UNPACKING_BLOCKERS_2026-06-14.md` |
| Interfaces, traits, implementation checks, anonymous classes | 78 combined | `PHPT_BROAD_1K_CLASS_DECLARATION_FRONTIER_2026-06-14.md` |
| Static locals, nullable type hints, variable variables | 33 combined | `PHPT_BROAD_1K_FUNCTION_DYNAMIC_TYPE_BLOCKERS_2026-06-14.md` |
| Named internal-call arguments | 1 | `PHPT_BROAD_1K_ARRAY_NAMED_CALLBACK_FRONTIER_2026-06-14.md` |
| Generators/yield | 1 | no focused broad 1k generator map in this branch |

The `ext/standard/tests/array/` rows in this bucket are language-gated before
array helper behavior can be measured:

```text
ext/standard/tests/array/GHSA-h96m-rvf9-jgm2.phpt
ext/standard/tests/array/array_combine.phpt
ext/standard/tests/array/array_diff_max_elements.phpt
ext/standard/tests/array/array_diff_uassoc_basic.phpt
ext/standard/tests/array/array_fill_object.phpt
ext/standard/tests/array/array_filter_invalid_mode.phpt
ext/standard/tests/array/array_filter_object.phpt
ext/standard/tests/array/array_find_types.phpt
ext/standard/tests/array/array_intersect_uassoc_basic.phpt
ext/standard/tests/array/array_map_object1.phpt
ext/standard/tests/array/array_push_empty.phpt
```

## Why This Is Not A Narrow Fix

The largest single group, PHP attributes, needs lexer/parser support for
attribute groups, AST storage on every attachable declaration, constant
expression evaluation for attribute arguments, delayed validation, built-in
attribute classes, and Reflection APIs. The next largest groups are similarly
cross-cutting: unpacking needs source-order argument expansion and array spread
semantics; class declaration rows need interface, trait, anonymous-class, and
implementation-check metadata; function dynamic rows need static local storage,
nullable type checks, and runtime symbol-table lookup.

Treating these rows as runnable before those generic layers exist would turn
missing semantic frontiers into noisy failures. The credible next movement is
to split and implement one surface at a time, then reclassify this exact broad
1k manifest.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-a0r0-final
```

Expected evidence from this slice:

- Broad 1k classify-only: 1,000 selected, 424 runnable, 576 excluded.
- Unsupported-language bucket: 288 rows.
- Largest unsupported-language blocker: 141 attribute syntax/reflection rows.
