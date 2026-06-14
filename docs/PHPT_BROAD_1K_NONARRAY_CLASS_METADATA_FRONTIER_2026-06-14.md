# PHPT Broad 1k Non-Array Class Metadata Frontier: 2026-06-14

Issue: `ptn-oe6i`

This broad PHPT slice maps the 1k rows blocked on class/object metadata outside
the standard-array object frontier. It is a blocker map, not a support claim:
these rows require generic class-table, property, method, reflection, autoload,
and magic-dispatch semantics before they can become useful executable PHPT
signal.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-oe6i-baseline-rebased
```

Generated manifest:
`.runtime/ptn-oe6i-baseline-rebased/20260614T045037Z/phpt-baseline-1000.txt`

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T045037Z.tsv
.runtime/phpt-progress/runnable-20260614T045037Z.txt
.runtime/phpt-progress/summary-20260614T045037Z.txt
```

PTN state: rebased `ptn-oe6i` branch after `ptn-oiin`.

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 429 | 571 |

Top broad classifier buckets:

| Bucket | Rows |
| --- | ---: |
| `unsupported-language` | 281 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |

## Focused Frontier

Committed manifest:
`tools/phpt-nonarray-class-metadata-frontier-manifest.txt`

Selection from the classifier:

```sh
awk -F'\t' '$2=="unsupported-class-metadata" && $1 !~ /^ext\/standard\/tests\/array\// {print $1}' \
  .runtime/phpt-progress/classification-20260614T045037Z.tsv
```

Focused classifier result:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-nonarray-class-metadata-frontier-manifest.txt
```

Latest focused result at `.runtime/phpt-progress/summary-20260614T055231Z.txt`:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 74 | 0 | 74 |

All 74 focused rows are Zend rows that remain excluded. The current focused
classifier split is 73 `unsupported-class-metadata` rows and 1
`unsupported-language` row.

## Reason Split

| Blocker | Rows |
| --- | ---: |
| Typed property metadata | 12 |
| Non-public property visibility metadata | 10 |
| Runtime class autoload symbol-table mutation | 9 |
| Unsupported magic method dispatch/reflection metadata | 9 |
| Internal attribute/reflection metadata | 8 |
| Indirect readonly property mutation diagnostics | 7 |
| Non-public method visibility dispatch and diagnostics | 6 |
| Abstract class/method contract metadata | 5 |
| Final class/method override metadata | 4 |
| Complete internal arginfo/class registry reflection | 3 |
| `ReflectionFunction::getClosureThis()` closure binding metadata | 1 |

## Path Concentration

| Path family | Rows |
| --- | ---: |
| `Zend/tests/access_modifiers` | 13 |
| `Zend/tests/asymmetric_visibility` | 16 |
| `Zend/tests/attributes` | 8 |
| `Zend/tests/autoload` | 9 |
| Other `Zend/tests` rows | 28 |

Representative rows:

```text
Zend/tests/abstract-static.phpt
Zend/tests/access_modifiers/access_modifiers_008.phpt
Zend/tests/assign_typed_ref_result.phpt
Zend/tests/asymmetric_visibility/static_props.phpt
Zend/tests/attributes/029_reflect_internal_symbols.phpt
Zend/tests/autoload/bug61011.phpt
Zend/tests/backtrace/debug_backtrace_options.phpt
Zend/tests/bug30140.phpt
Zend/tests/bug38779.phpt
```

## Why This Is A Blocker

The shared dependency is the object/class metadata model, not individual test
shapes. Reopening this frontier requires generic compiler and runtime work:

- class-table metadata for abstract/final contracts, typed properties,
  readonly/asymmetric property state, non-public visibility, and method
  visibility;
- autoload and runtime symbol-table mutation boundaries that can change class
  availability after initial compilation;
- reflection metadata for internal classes, functions, attributes, closure
  binding, properties, and arginfo;
- magic method dispatch integrated with visibility, backtrace, property access,
  and object conversion;
- diagnostics that report the same fatal/warning boundary across direct calls,
  property access, reflection, and callback dispatch.

Treating these rows as runnable today would turn absent metadata layers into
noisy parser/runtime failures. Keeping them mapped as a 74-row non-array class
metadata frontier makes the broad 1k dashboard more actionable until the
generic class model grows those semantics.

## Verification

```sh
cargo fmt --check
cargo test --test phpt_classifier
tools/run-phpt-baseline.sh --tier 1000 --classify-only --out-dir .runtime/ptn-oe6i-baseline-rebased
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs tools/phpt-nonarray-class-metadata-frontier-manifest.txt
```
