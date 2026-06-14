# PHPT Broad 1k Post-Language Split Transitional Map: 2026-06-14 ptn-h5qy

Issue: `ptn-h5qy`

This slice records the broad PHPT 1k classifier on artifact commit
`79945bc00ea9` after the language classifier split, but before later classifier
refinements such as the `ptn-0fmh` class-metadata split. It is a transitional
blocker map and coverage check, not a runtime behavior change. The broad
runnable surface was already covered by committed focused manifests, and the
remaining 25+ row clusters were semantic frontiers that crossed parser,
runtime, metadata, request, or diagnostic boundaries.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h5qy-baseline
```

Generated broad manifests:

```text
.runtime/ptn-h5qy-baseline/20260614T095243Z/phpt-baseline-1000.txt
.runtime/ptn-h5qy-baseline/20260614T095243Z/phpt-baseline-5000.txt
.runtime/ptn-h5qy-baseline/20260614T095243Z/phpt-baseline-10000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T095244Z.tsv
.runtime/phpt-progress/runnable-20260614T095244Z.txt
.runtime/phpt-progress/excluded-20260614T095244Z.tsv
```

Recorded state:

```text
PTN artifact commit: 79945bc00ea9
php-src PHPT corpus: /home/claude/php-src-phpt
corpus revision: 8c63ec400ce8e07c57a8d9499317b96a8beafb8b
```

Classifier result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 424 | 576 |

Classifier split:

| Classification | Rows |
| --- | ---: |
| `runnable` | 424 |
| `unsupported-attribute-metadata` | 149 |
| `unsupported-class-metadata` | 135 |
| `unsupported-class-declaration` | 78 |
| `unsupported-call-unpacking` | 34 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `unsupported-type-hint` | 14 |
| `sapi-behavior` | 13 |
| `unsupported-function-state` | 11 |
| `unsupported-assertion-runtime` | 9 |
| `unsupported-dynamic-symbol` | 8 |
| `unsupported-diagnostics-ini` | 5 |
| `harness-cleanup` | 4 |
| `process-boundary` | 3 |
| `skipif-precondition` | 2 |
| `unsupported-function-disable-ini` | 2 |
| `unsupported-host-path-ini` | 2 |
| `unsupported-opcache-ini` | 2 |
| `unsupported-scalar-format-ini` | 2 |
| `environment-assumption` | 1 |
| `external-service` | 1 |
| `unsupported-generator-runtime` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-internal-call-binding` | 1 |
| `unsupported-resource-limit` | 1 |

## Runnable Coverage Check

The 424 recorded broad-runnable rows were all present in committed focused
manifests:

```sh
comm -23 \
  <(LC_ALL=C sort -u .runtime/phpt-progress/runnable-20260614T095244Z.txt) \
  <(awk 'NF && $1 !~ /^#/ {print $1}' tools/phpt-*-manifest.txt | LC_ALL=C sort -u) \
  | wc -l
```

Result:

```text
0
```

After rebasing over newer focused frontier manifests, the committed focused
manifest inventory contains 1,655 unique rows.

Runnable rows by source family:

| Source family | Rows |
| --- | ---: |
| `ext/standard/tests/array/*` | 294 |
| root-level `Zend/tests/*.phpt` | 81 |
| `Zend/tests/asymmetric_visibility/*` | 22 |
| `tests/basic/*` | 16 |
| `Zend/tests/ast/*` | 4 |
| `Zend/tests/arrow_functions/*` | 3 |
| `Zend/tests/assert/*` | 2 |
| `Zend/tests/access_modifiers/*` | 1 |
| `Zend/tests/attributes/*` | 1 |

## Focused Language/Runtime Blockers

The recorded post-split language/runtime frontier contains 147 rows. Focused
manifest:

```text
.runtime/ptn-h5qy/language-runtime-frontier.txt
```

Selection:

```sh
awk -F '\t' '$2 ~ /^unsupported-(class-declaration|call-unpacking|type-hint|function-state|dynamic-symbol|generator-runtime|internal-call-binding)$/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T095244Z.tsv \
  > .runtime/ptn-h5qy/language-runtime-frontier.txt
```

Focused replay:

```sh
PHPT_PROGRESS_DIR=.runtime/ptn-h5qy-language-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-h5qy/language-runtime-frontier.txt
```

Focused artifacts:

```text
.runtime/ptn-h5qy-language-focused/classification-20260614T095824Z.tsv
.runtime/ptn-h5qy-language-focused/runnable-20260614T095824Z.txt
.runtime/ptn-h5qy-language-focused/excluded-20260614T095824Z.tsv
```

Focused result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 147 | 0 | 147 |

Focused split:

| Classification | Rows | Generic frontier |
| --- | ---: | --- |
| `unsupported-class-declaration` | 78 | Interfaces, traits, implementation checks, and anonymous classes need class-table graph and declaration diagnostics. |
| `unsupported-call-unpacking` | 34 | Call-site spread and array unpacking need AST/IR representation, ordered argument expansion, by-reference binding, and array-spread key semantics. |
| `unsupported-type-hint` | 14 | Nullable and `never` type metadata need coercion, return validation, and control-flow diagnostics. |
| `unsupported-function-state` | 11 | Function-local `static` storage needs per-function state, initializer timing, references, recursion, and shutdown behavior. |
| `unsupported-dynamic-symbol` | 8 | Variable variables need an explicit runtime symbol-table/fallback boundary. |
| `unsupported-generator-runtime` | 1 | Generator/yield lowering needs suspension runtime and return/reference boundaries. |
| `unsupported-internal-call-binding` | 1 | Named arguments for modeled internals need parameter metadata and native binding. |

## Large Metadata Blockers

At that point, the 25+ row blockers outside the language/runtime focused set
were also generic semantic systems:

| Rows | Blocker |
| ---: | --- |
| 149 | PHP attribute syntax, reflection metadata, and internal attribute metadata. |
| 135 | Class/object metadata: magic dispatch/reflection, visibility metadata, typed properties, autoload, readonly diagnostics, abstract/final contracts, and internal arginfo. |
| 28 | Request/input INI state such as upload, argv, post body, and variable-order boundaries. |

Representative metadata split from that broad classifier:

| Rows | Reason |
| ---: | --- |
| 141 | PHP attribute syntax and reflection metadata. |
| 69 | Magic method dispatch/reflection metadata. |
| 25 | Trait declarations. |
| 23 | Interface declarations. |
| 19 | Non-public property visibility metadata. |
| 15 | Interface implementation checks. |
| 15 | Anonymous class syntax. |
| 12 | Typed property metadata. |
| 9 | Runtime class autoload symbol-table mutation. |
| 8 | Internal attribute/reflection metadata. |

## Implementation Boundary

No single coherent implementation patch in this transitional broad 1k slice
reached the 25-row target without crossing multiple generic subsystems:

1. The 424 runnable rows are already covered by committed focused manifests.
2. The largest runnable family, `ext/standard/tests/array/*`, is a regression
   target whose residual failures are split across key/value conversion,
   callback dispatch, ordered mutation/reference behavior, random selection,
   and comparator semantics.
3. The newly split language/runtime rows are clearer than the old aggregate
   `unsupported-language` bucket, but each category still needs a dedicated
   parser/runtime design before it should become runnable.
4. Attribute, class/object metadata, and request/SAPI rows are high-count
   blockers, but reopening them before the shared metadata or request
   boundaries exist would only convert explicit classifiers into noisy native
   failures.

Next credible implementation splits:

1. Class-table declaration graph for interfaces, traits, implementation checks,
   and anonymous class metadata.
2. Call-site and array unpacking AST/IR plus argument/array expansion runtime.
3. Nullable/`never` type metadata and function-local static storage as separate
   parser/runtime slices.
4. Shared object metadata for magic dispatch, visibility, typed properties, and
   reflection-facing metadata.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-h5qy-baseline
PHPT_PROGRESS_DIR=.runtime/ptn-h5qy-language-focused \
  tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  .runtime/ptn-h5qy/language-runtime-frontier.txt
```

Expected evidence:

- Broad 1k classify-only: 1,000 selected, 424 runnable, 576 excluded.
- Focused language/runtime classify-only: 147 selected, 0 runnable, 147 excluded.
- Runnable coverage delta against committed focused manifests: 0 unmatched rows.
