# PHPT Broad 1k Attribute/Metadata Frontier: 2026-06-14

Issue: `ptn-lzef`

This slice starts from the broad 1k PHPT baseline on `origin/master` and maps
the current `Zend/tests/attributes/*` cluster. It is a blocker map, not an
attribute support claim: moving this cluster needs coordinated parser, AST,
metadata, validation, Reflection, internal-class, and diagnostics work.

## Broad 1k Evidence

Source state:

- PTN: `62577c15c2f7`
- php-src PHPT corpus: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-lzef-baseline-before
```

Generated broad manifest:

```text
.runtime/ptn-lzef-baseline-before/20260614T054332Z/phpt-baseline-1000.txt
```

Classifier artifacts:

```text
.runtime/phpt-progress/classification-20260614T054332Z.tsv
.runtime/phpt-progress/runnable-20260614T054332Z.txt
.runtime/phpt-progress/excluded-20260614T054332Z.tsv
```

Observed broad 1k classifier result from the generated artifacts:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1,000 | 428 | 572 |

Top broad classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 282 |
| `unsupported-class-metadata` | 144 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 18 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |

The baseline wrapper was stopped after the classifier artifacts were written
because the shell wrapper waited silently in post-processing. The focused
cluster validation below completed normally and is the committed reproduction
path for this slice.

## Focused Attribute Cluster

Focused manifest:

```text
tools/phpt-attributes-metadata-frontier-manifest.txt
```

It is a mechanical extraction of the 204 broad 1k rows whose path starts with
`Zend/tests/attributes/`.

Validation command:

```sh
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attributes-metadata-frontier-manifest.txt
```

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T055056Z.tsv
.runtime/phpt-progress/runnable-20260614T055056Z.txt
.runtime/phpt-progress/excluded-20260614T055056Z.tsv
```

Result:

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 204 | 1 | 203 |

Focused classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 187 |
| `unsupported-class-metadata` | 8 |
| `unsupported-extension` | 8 |
| runnable | 1 |

Reason split:

| Generic blocker | Rows |
| --- | ---: |
| PHP attribute syntax (`#[...]`) plus declaration/reflection metadata | 141 |
| Interface declaration metadata | 17 |
| Trait declaration metadata | 15 |
| Nullable type-hint metadata and coercion | 12 |
| Internal attribute/reflection metadata (`Reflection*::getAttributes()`, `Attribute`, `Deprecated`, `NoDiscard`) | 8 |
| Missing `zend_test` extension | 7 |
| Interface implementation checks | 2 |
| Missing `opcache` extension | 1 |
| Runnable but failing native NoDiscard/DateTime metadata row | 1 |

Path split:

| Attribute subdirectory | Rows |
| --- | ---: |
| `override/` | 50 |
| `deprecated/` | 47 |
| root `Zend/tests/attributes/*.phpt` | 39 |
| `nodiscard/` | 25 |
| `delayed_target_validation/` | 20 |
| `constants/` | 19 |
| `Attribute/` | 4 |

The single runnable row is:

```text
Zend/tests/attributes/nodiscard/005.phpt
```

Focused execution:

```sh
tools/run-bounded-phpt.sh .runtime/phpt-progress/runnable-20260614T055056Z.txt
```
Result:

| Selected | Runnable | Passed | Failed |
| ---: | ---: | ---: | ---: |
| 1 | 1 | 0 | 1 |

The failure is generic metadata/runtime coverage, not expected-output shape:

```text
Fatal error: Class "DateTimeImmutable" not found
```

The expected row behavior is a `NoDiscard` warning on a native
`DateTimeImmutable::setTimestamp()` method call, so this row depends on both
DateTime internal class metadata and native NoDiscard attribute diagnostics.

## Why This Is Not A Narrow Fix

The 204-row cluster crosses several architecture boundaries:

- Lexer/parser support for attribute groups before every attachable
  declaration and parameter.
- AST storage for grouped attributes, namespaced names, arguments, source
  spans, and attachment targets.
- Constant-expression evaluation for attribute arguments.
- Metadata tables for functions, closures, methods, parameters, classes,
  properties, class constants, and global constants.
- Attribute target, repeatability, delayed validation, and built-in attribute
  semantics for `Attribute`, `Deprecated`, `Override`, `NoDiscard`, and
  `AllowDynamicProperties`.
- Reflection APIs, especially `ReflectionAttribute` and `getAttributes()` on
  all modeled symbols.
- Integration with still-bounded interface, trait, nullable type-hint,
  internal-extension, DateTime, and native-method metadata.

Until those pieces exist as reusable compiler/runtime metadata, keeping the
cluster classified is higher-signal than letting 203 parser/metadata rows fail
as noisy runnable tests.

## Suggested Implementation Splits

1. Parse and store inert attribute groups on declarations without runtime
   validation; keep Reflection access classified until metadata is complete.
2. Add `Attribute` class constants and target/repeatability validation over the
   stored metadata.
3. Add `ReflectionAttribute` and `getAttributes()` for functions, classes,
   methods, parameters, properties, and constants.
4. Integrate built-in attributes such as `Deprecated`, `Override`,
   `NoDiscard`, and `AllowDynamicProperties` with diagnostics and internal
   method metadata.

## Verification

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-lzef-baseline-before
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-attributes-metadata-frontier-manifest.txt
tools/run-bounded-phpt.sh .runtime/phpt-progress/runnable-20260614T055056Z.txt
```
