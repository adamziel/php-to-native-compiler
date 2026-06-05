# 221205Z Direct FAILED/BORKED Regression Triage

Owner: developer-407
Lane: 68
Date: 2026-06-05

## Scope

This is the read-only triage for the `27` direct `FAILED` rows and `3` direct
`BORKED` rows in the blocked `221205Z` public PHPT candidate gate. No
compiler/runtime files were edited and no full PHPT gate was run.

Candidate gate directory:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Accepted baseline directory:

`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`

PHP source checkout:

`/home/claude/php-src-phpt` at php-src pin
`f97ff597429a2fe633665a7e02d97c8077f9f90f`.

## Result

The direct non-PASS regression rows are confirmed as:

| Bucket | Rows | Interpretation |
| --- | ---: | --- |
| Readonly/internal property diagnostics and property-hook/interface metadata | 15 | Highest-yield direct semantic bucket. These rows are about internal readonly property write/unset wording, readonly class/trait property semantics, or property hooks satisfying interface properties. |
| Object lifecycle, destructor timing, iterator lifetime, closure-cycle GC | 6 | Direct semantic bucket, likely runtime/object-lifecycle work after focused replay. |
| Assertion, throwable stringification, serialization, uncaught throwable behavior | 4 | Direct semantic bucket, but split after lifecycle because several rows may depend on throwable formatting or exception object state. |
| Internal inheritance diagnostic ordering | 1 | Single diagnostic row; replay before opening a broad inheritance lane. |
| Opcache SCCP memory-leak PHPT | 1 | Single opcache-specific row; isolate from general compatibility work. |
| SKIPIF extension/core constants | 3 | BORKED before body execution; wrapper/environment constant exposure, not product body semantics. |
| Total | 30 | These are separate from the `1136` absent-result rows. |

The candidate public score must not move. These `30` direct rows need narrow
repair or adjudication, and the `1136` absent rows still need control-plane
completion/replay before they can be treated as semantic failures.

## Evidence Table

| Status | Row | Title | Evidence | Bucket | Recommendation |
| --- | --- | --- | --- | --- | --- |
| `FAILED` | `php-src/ext/bcmath/tests/number/properties_unset.phpt` | `BcMath\Number properties unset` | `shard-03/stdout.log`; `shard-03/results.txt`; no `shard-03/run-tests.log` preserved | Readonly/internal property diagnostics | Replay one `BcMath\Number` property row with diff capture, then fix readonly unset diagnostics if reproduced. |
| `FAILED` | `php-src/ext/bcmath/tests/number/properties_write_error.phpt` | `BcMath\Number properties write error` | `shard-04/stdout.log`; `shard-04/results.txt`; no `shard-04/run-tests.log` preserved | Readonly/internal property diagnostics | Replay one `BcMath\Number` property row with diff capture, then fix readonly write diagnostics if reproduced. |
| `FAILED` | `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt` | `DatePeriod modify readonly property` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | Readonly/internal property diagnostics | Include in first readonly/internal property repair lane; existing report evidence says actual wording includes `protected(set) readonly property ... from global scope` variants. |
| `FAILED` | `php-src/ext/date/tests/DatePeriod_properties2.phpt` | `DatePeriod: Test cannot modify read only properties` | `shard-06/run-tests.log`; `shard-06/stdout.log`; `shard-06/results.txt` | Readonly/internal property diagnostics | Same lane as `DatePeriod_modify_readonly_property.phpt`; verify all `DatePeriod` readonly diagnostics from one focused replay. |
| `FAILED` | `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt` | `Changing Directory::$handle property` | `shard-06/run-tests.log`; `shard-06/stdout.log`; `shard-06/results.txt` | Readonly/internal property diagnostics | Pair with `DirectoryClass_readonly_path.phpt`; repair internal `Directory` readonly property metadata/diagnostics or adjudicate explicitly. |
| `FAILED` | `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt` | `Changing Directory::$handle property` | `shard-01/run-tests.log`; `shard-01/stdout.log`; `shard-01/results.txt` | Readonly/internal property diagnostics | Pair with `DirectoryClass_readonly_handle.phpt`; note title appears to mention handle even for the path row. |
| `FAILED` | `php-src/ext/xmlreader/tests/014.phpt` | `XMLReader: libxml2 XML Reader, read-only element values cannot be modified` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | Readonly/internal property diagnostics | Same readonly diagnostic repair lane; existing report evidence says actual wording includes `protected(set) readonly property ... from global scope`. |
| `FAILED` | `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt` | `#[\Deprecated]: Deprecated::$message is readonly.` | `shard-01/run-tests.log`; `shard-01/stdout.log`; `shard-01/results.txt` | Readonly/internal property diagnostics | Include in readonly diagnostic precheck with one internal extension row to prove core and internal-object coverage. |
| `FAILED` | `php-src/Zend/tests/attributes/deprecated/property_readonly_002.phpt` | `#[\Deprecated]: Deprecated::$since is readonly.` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | Readonly/internal property diagnostics | Same `Deprecated` attribute property diagnostic family; do not split into a separate lane unless focused replay diverges. |
| `FAILED` | `php-src/Zend/tests/attributes/override/properties_08.phpt` | `#[\Override]: Properties: On used trait with interface property.` | `shard-01/run-tests.log`; `shard-01/stdout.log`; `shard-01/results.txt` | Property-hook/interface metadata | Split from pure readonly wording if repair scope grows; expected trait property should satisfy interface property with `#[Override]`. |
| `FAILED` | `php-src/Zend/tests/property_hooks/gh19548.phpt` | `GH-19548: Segfault when using inherited properties and opcache` | `shard-03/stdout.log`; `shard-03/results.txt`; no `shard-03/run-tests.log` preserved | Property-hook/interface metadata | Replay with `gh19548_002.phpt`; existing report says candidate requires hooked interface gets that inherited concrete properties should satisfy. |
| `FAILED` | `php-src/Zend/tests/property_hooks/gh19548_002.phpt` | `GH-19548: Segfault when using inherited properties and opcache (multiple properties)` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | Property-hook/interface metadata | Best preserved property-hook/interface precheck row because `run-tests.log` exists. |
| `FAILED` | `php-src/Zend/tests/readonly_classes/readonly_class_property1.phpt` | `Normal properties of a readonly class are implicitly declared as readonly` | `shard-03/stdout.log`; `shard-03/results.txt`; no `shard-03/run-tests.log` preserved | Readonly class property semantics | Replay after readonly diagnostic lane prechecks; may require readonly class metadata, not only diagnostic wording. |
| `FAILED` | `php-src/Zend/tests/readonly_classes/readonly_class_property2.phpt` | `Promoted properties of a readonly class are implicitly declared as readonly` | `shard-04/stdout.log`; `shard-04/results.txt`; no `shard-04/run-tests.log` preserved | Readonly class property semantics | Pair with `readonly_class_property1.phpt`; needs replay with diff capture before implementation. |
| `FAILED` | `php-src/Zend/tests/readonly_props/readonly_trait_mismatch.phpt` | `Readonly mismatch of imported trait properties` | `shard-04/stdout.log`; `shard-04/results.txt`; no `shard-04/run-tests.log` preserved | Readonly trait/property compatibility | Separate from internal-object diagnostics if replay shows trait-composition metadata failure. |
| `FAILED` | `php-src/tests/classes/ctor_dtor.phpt` | `ZE2 The new constructor/destructor is called` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | Object lifecycle/destructor timing | First lifecycle precheck; existing report says destructor output is emitted too early. |
| `FAILED` | `php-src/tests/classes/destructor_and_echo.phpt` | `ZE2 Destructors and echo` | `shard-04/stdout.log`; `shard-04/results.txt`; no `shard-04/run-tests.log` preserved | Object lifecycle/destructor timing | Replay with diff capture after `ctor_dtor.phpt`; avoid broad lifecycle work until the exact order is reproduced. |
| `FAILED` | `php-src/tests/classes/factory_and_singleton_002.phpt` | `ZE2 factory and singleton, test 2` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | Object lifecycle/destructor visibility | Existing report says candidate fatals on protected destructor access; pair with destructor timing precheck. |
| `FAILED` | `php-src/tests/classes/iterators_002.phpt` | `ZE2 iterators and break` | `shard-01/run-tests.log`; `shard-01/stdout.log`; `shard-01/results.txt` | Iterator object lifetime | Existing report says inner iterator destructor runs too early; include if lifecycle lane touches object lifetime. |
| `FAILED` | `php-src/Zend/tests/bug73989.phpt` | `Bug #73989 (PHP 7.1 Segfaults within Symfony test suite)` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | Object lifecycle/closure cycle cleanup | Replay after basic destructor rows; PHPT repeatedly creates cyclic objects whose destructor invokes a captured closure. |
| `FAILED` | `php-src/Zend/tests/gc/bug63635.phpt` | `Bug #63635 (Segfault in gc_collect_cycles)` | `shard-06/run-tests.log`; `shard-06/stdout.log`; `shard-06/results.txt` | Object lifecycle/GC | Keep as GC stress follow-up, not the first lifecycle fix, unless replay shows the same root as destructor ordering. |
| `FAILED` | `php-src/Zend/tests/assert/expect_008.phpt` | `test disabled expectations have no ill side effects` | `shard-04/stdout.log`; `shard-04/results.txt`; no `shard-04/run-tests.log` preserved | Assertion side effects | Replay with diff capture before implementation; one of the rows without preserved diff. |
| `FAILED` | `php-src/Zend/tests/assert/expect_011.phpt` | `test overloaded __toString on custom exception` | `shard-01/run-tests.log`; `shard-01/stdout.log`; `shard-01/results.txt` | Assertion/throwable stringification | Existing report says candidate reports `undefined property MyExpectations::$string` instead of expected `AssertionError` message; narrow assertion/throwable lane. |
| `FAILED` | `php-src/Zend/tests/serialize/bug76502.phpt` | `Bug #76502: Chain of mixed exceptions and errors does not serialize properly` | `shard-03/stdout.log`; `shard-03/results.txt`; no `shard-03/run-tests.log` preserved | Throwable serialization | Replay with diff capture; likely separate from assertion callback behavior. |
| `FAILED` | `php-src/Zend/tests/uncaught_exception_error_supression.phpt` | `Error suppression should have no impact on uncaught exceptions` | `shard-03/stdout.log`; `shard-03/results.txt`; no `shard-03/run-tests.log` preserved | Uncaught throwable formatting | Replay with diff capture; can be grouped with throwable formatting after assertion row evidence. |
| `FAILED` | `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt` | `Test unresolvable inheritance check due to unavailable parameter type when the parent has a tentative return type.` | `shard-01/run-tests.log`; `shard-01/stdout.log`; `shard-01/results.txt` | Internal inheritance diagnostic ordering | One-row diagnostic lane only if replay still fails; do not generalize to all variance rows. |
| `FAILED` | `php-src/ext/opcache/tests/opt/sccp_037.phpt` | `SCCP 037: Memory leak` | `shard-04/stdout.log`; `shard-04/results.txt`; no `shard-04/run-tests.log` preserved | Opcache memory-leak PHPT | Deprioritize behind multi-row buckets; isolate as one-row opcache leak replay/adjudication. |
| `BORKED` | `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` | `Test IntlNumberRangeFormatter::createFromSkeleton throws error for ICU < 63` | `shard-06/run-tests.log`; `shard-06/stdout.log`; `shard-06/results.txt` | SKIPIF constant exposure | Fix wrapper/SKIPIF environment for `INTL_ICU_VERSION`, or delegate SKIPIF to compatible system PHP; no body semantics proven. |
| `BORKED` | `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt` | `openssl.libctx INI setting when Argon2 disable or ZTS not used` | `shard-02/run-tests.log`; `shard-02/stdout.log`; `shard-02/results.txt` | SKIPIF constant exposure | Fix wrapper/SKIPIF environment for `ZEND_THREAD_SAFE`; no body semantics proven. |
| `BORKED` | `php-src/ext/pcre/tests/grep2.phpt` | `preg_grep() 2nd test` | `shard-05/run-tests.log`; `shard-05/stdout.log`; `shard-05/results.txt` | SKIPIF constant exposure | Fix wrapper/SKIPIF environment for `PCRE_JIT_SUPPORT`; no body semantics proven. |

## Repair and Adjudication Plan

1. Open the first direct repair lane for readonly/internal property diagnostics
   and property metadata. Use a small precheck set:
   `DatePeriod_modify_readonly_property.phpt`,
   `DirectoryClass_readonly_handle.phpt`,
   `property_readonly_001.phpt`,
   `gh19548_002.phpt`,
   `properties_write_error.phpt`, and `014.phpt`.
   Split property-hook/interface compatibility out if it requires different
   metadata than readonly write/unset diagnostics.

2. Open a separate SKIPIF environment lane for the three `BORKED` rows. These
   fail before the test body because skip scripts reference missing constants:
   `INTL_ICU_VERSION`, `ZEND_THREAD_SAFE`, and `PCRE_JIT_SUPPORT`. This should
   be fixed or adjudicated at the PHPT wrapper/environment layer, not in the
   compiler runtime.

3. Open a lifecycle/destructor replay lane after the readonly lane has an
   owner. Start with `ctor_dtor.phpt`, `factory_and_singleton_002.phpt`, and
   `iterators_002.phpt`, because they have preserved `run-tests.log` evidence.
   Add `bug73989.phpt` and `bug63635.phpt` only if the same lifetime root is
   reproduced.

4. Open assertion/throwable work only after lifecycle replay confirms it is
   independent. Use `expect_011.phpt` first because it has preserved
   `run-tests.log` evidence. Replay `expect_008.phpt`, `bug76502.phpt`, and
   `uncaught_exception_error_supression.phpt` for diff capture before repair.

5. Treat
   `type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt`
   and `ext/opcache/tests/opt/sccp_037.phpt` as one-row lanes unless focused
   replay finds broader clusters. Do not let either block the higher-yield
   direct buckets above.

## Commands Used

No full gate was run. Commands were artifact reads, report reads, and small
Python text-parsing scripts:

```sh
sed -n '1,260p' .harness/reports/221205Z-pass-regression-manifest.md
sed -n '100,210p' .harness/reports/221205Z-regression-status-summary-refresh-dev313.md
sed -n '96,124p' .harness/reports/221205Z-unsupported-boundary-overlap.md
rg -n "readonly|BORKED|FAILED|DirectoryClass|DatePeriod|bcmath|XMLReader|opcache|ctor_dtor|expect_008|bug63635|bug76502|iterators_002|grep2|rangeformatter|openssl" \
  .harness/reports/221205Z-source-diff-risk.md \
  .harness/reports/221205Z-zend-classes-sapi.md \
  .harness/reports/221205Z-standard-filesystem-http.md \
  .harness/reports/221205Z-secondary-ext.md

python3 - <<'PY'
from pathlib import Path
C = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
reg = set((C / 'regressions-from-latest-published-passes.txt').read_text().splitlines())
direct = []
for line in (C / 'current-status.normalized.tsv').read_text(errors='replace').splitlines():
    status, path = line.split('\t', 1)
    if path in reg and status in {'FAILED', 'BORKED'}:
        direct.append((status, path))
print(len(direct))
for row in sorted(direct):
    print(row)
PY
```

## Notes

- `DEVELOPMENT.md` was requested by the harness prompt but is not present in
  this checkout; `find .. -name DEVELOPMENT.md -print` returned no match.
- The direct rows are not evidence that eval or variable-variable support
  should be prioritized. The current goal keeps those late-priority.
- Rows from shards `03` and `04` often have `stdout.log`/`results.txt` only
  because those shards lack preserved `run-tests.log`; replay with diff capture
  is required before implementing those row-specific fixes.
