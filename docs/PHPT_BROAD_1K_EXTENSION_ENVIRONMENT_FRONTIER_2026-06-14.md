# PHPT Broad 1k Extension/Environment Frontier: 2026-06-14

Issue: `ptn-rdce`

This slice refreshes broad 1k PHPT evidence on `origin/master` and maps the
remaining rows blocked by execution environment availability or adjacent runtime
boundaries rather than local PHP compiler semantics: unavailable PHP extensions,
child-process execution, external php-src service harnesses, host
preconditions, PHPT `--ENV--` setup, unsupported internal helper semantics, and
resource-limit diagnostics.

This is a blocker map, not a support claim. Reopening these rows requires
generic extension modules, native process boundaries, harness/runtime
environment support, or shared internal/resource-limit runtime layers. They
should not become row-local expected-output patches.

## Broad 1k Evidence

Command:

```sh
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-rdce-before
```

Generated manifest:
`.runtime/ptn-rdce-before/20260614T070759Z/phpt-baseline-1000.txt`

Artifacts:

```text
.runtime/phpt-progress/classification-20260614T070759Z.tsv
.runtime/phpt-progress/runnable-20260614T070759Z.txt
.runtime/phpt-progress/summary-20260614T070759Z.txt
```

PTN state: `540e57ee9555`

Corpus revision: `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 1000 | 424 | 576 |

Top classifier buckets:

| Classification | Rows |
| --- | ---: |
| `unsupported-language` | 288 |
| `unsupported-class-metadata` | 143 |
| `unsupported-request-input-ini` | 28 |
| `unsupported-extension` | 20 |
| `unsupported-diagnostics-runtime` | 17 |
| `unsupported-assertion-ini` | 17 |
| `unsupported-resource-limit-ini` | 15 |
| `sapi-behavior` | 13 |
| `unsupported-assertion-runtime` | 9 |

## Focused Frontier

Committed manifest:
`tools/phpt-extension-environment-frontier-manifest.txt`

Selection from the broad classification:

```sh
awk -F'\t' '$2 ~ /^(unsupported-extension|process-boundary|external-service|environment-assumption|skipif-precondition|unsupported-internal|unsupported-resource-limit)$/ {print $1}' \
  .runtime/phpt-progress/classification-20260614T070759Z.tsv
```

Focused classify-only result:

```text
.runtime/phpt-progress/classification-20260614T073609Z.tsv
.runtime/phpt-progress/runnable-20260614T073609Z.txt
.runtime/phpt-progress/summary-20260614T073609Z.txt
```

| Selected | Runnable | Excluded |
| ---: | ---: | ---: |
| 29 | 0 | 29 |

Classifier split:

| Classification | Rows |
| --- | ---: |
| `unsupported-extension` | 20 |
| `process-boundary` | 3 |
| `skipif-precondition` | 2 |
| `external-service` | 1 |
| `environment-assumption` | 1 |
| `unsupported-internal` | 1 |
| `unsupported-resource-limit` | 1 |

## Extension Split

The 20 unsupported-extension rows require extension modules that are not part
of PTN's current modeled extension set (`Core`, `date`, `pcre`, `standard`, and
`Reflection`):

| Extension | Rows |
| --- | ---: |
| `zend_test` | 10 |
| `mbstring` | 2 |
| `gmp` | 2 |
| `ffi` | 1 |
| `opcache` | 1 |
| `session` | 1 |
| `simplexml` | 1 |
| `xml` | 1 |
| `zlib` | 1 |

Rows:

```text
Zend/tests/GHSA-wm6j-2649-pv75.phpt
Zend/tests/attributes/016_custom_attribute_validation.phpt
Zend/tests/attributes/constants/ast_export.phpt
Zend/tests/attributes/constants/repeatable-internal.phpt
Zend/tests/attributes/delayed_target_validation/opcache_validator_errors.phpt
Zend/tests/attributes/nodiscard/006.phpt
Zend/tests/attributes/nodiscard/007.phpt
Zend/tests/attributes/nodiscard/009.phpt
Zend/tests/attributes/nodiscard/010.phpt
Zend/tests/backtrace/bug_debug_backtrace_replace_zend_execute_ex.phpt
Zend/tests/bug34199.phpt
Zend/tests/bug34617.phpt
ext/standard/tests/array/array_product_objects_operation_no_cast.phpt
ext/standard/tests/array/array_product_variation6.phpt
ext/standard/tests/array/array_sum_objects_operation_no_cast.phpt
ext/standard/tests/array/array_sum_objects_operation_no_cast_FFI.phpt
ext/standard/tests/array/array_sum_variation9.phpt
tests/basic/029.phpt
tests/basic/bug20539.phpt
tests/basic/req44164.phpt
```

`zend_test` rows are test-extension probes for internal attribute, backtrace,
object-operation, and validation behavior. They need a deliberate test
extension fixture or generic internal-extension loading model. The other
extensions require their own extension modules or a classified extension API
boundary, not ad hoc stubs.

## Runtime/Harness Environment Split

The remaining 9 rows are outside PHP extension availability:

| Surface | Rows | Row(s) |
| --- | ---: | --- |
| Child-process boundary | 3 | `tests/basic/GHSA-9pqp-7h25-4f32.phpt`, `tests/basic/bug71273.phpt`, `tests/basic/gh16998.phpt` |
| Static host preconditions | 2 | `Zend/tests/binary-32bit.phpt`, `tests/basic/consistent_float_string_casts.phpt` |
| External php-src service harness | 1 | `tests/basic/bug67198.phpt` |
| PHPT `--ENV--` setup | 1 | `tests/basic/gh7896.phpt` |
| Unsupported internal helper semantics | 1 | `Zend/tests/array_multisort_exception.phpt` |
| Resource-limit diagnostics | 1 | `ext/standard/tests/array/array_fill_error2.phpt` |

The child-process rows need the same native process boundary as the
filesystem/path/process `proc_*` frontier. The precondition rows are host gates
for 32-bit-only behavior and unavailable locale candidates. The service and
environment rows require harness support rather than compiler changes. The
`array_multisort()` row needs a shared multi-array by-reference sort layer, and
the huge `array_fill()` row needs bounded memory/resource-limit diagnostics
before either should be treated as a runnable compatibility row.

## Implementation Boundary

Future work should split this map along reusable architecture boundaries:

1. Extension module framework expansion for `mbstring`, `gmp`, `session`,
   `simplexml`, `xml`, `zlib`, `ffi`, and OPcache-adjacent metadata.
2. A deliberate `zend_test` compatibility fixture for php-src rows that probe
   engine-only test extension behavior.
3. Native process execution and pipe/resource support for the `process-boundary`
   rows.
4. PHPT harness modeling for external services, `--ENV--`, and static host
   preconditions.
5. Shared internal-helper and resource-limit runtime layers for
   `array_multisort()` and allocation-heavy array constructors.

Until those layers exist, these 29 rows should remain classified so broad PHPT
telemetry measures PTN's modeled PHP runtime rather than host availability.

## Verification

```sh
cargo fmt --check
tools/run-phpt-baseline.sh --tier 1000 --classify-only \
  --out-dir .runtime/ptn-rdce-before
tools/run-bounded-phpt.sh --classify-only --classify-harness-programs \
  tools/phpt-extension-environment-frontier-manifest.txt
```
