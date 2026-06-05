# 221205Z PASS-Regression Manifest

Run under review:
`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`

Accepted baseline:
`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`

## Result

The blocked 221205Z gate has exactly **1166** latest-published PASS regressions.

This matches both:

- `pass-regression-summary.tsv`: baseline passes `7869`, current passes `7196`, pass regressions `1166`.
- Set recomputation: `135138Z/current-passes.normalized.txt - 221205Z/current-passes.normalized.txt = 1166`.

The accepted public score remains `7873/20294` at `0b917f67`. The 221205Z candidate must remain blocked: its public-comparable artifact reports `7197/20294`, while the normalized unique PASS set used for regression accounting is `7196`.

## Accounting Caveats

The dominant regression symptom is not concrete PHPT failure output. It is missing candidate status rows.

| bucket | count | notes |
| --- | ---: | --- |
| Absent from candidate `current-status.normalized.tsv` and `all-results.txt` | 1136 | These rows were accepted PASS rows but have no candidate row/status. They account for 97.4% of the 1166 regressions. |
| Candidate `FAILED` | 27 | Concrete non-PASS rows with test titles in shard logs/stdout. |
| Candidate `BORKED` | 3 | Concrete SKIPIF failures from missing constants. |
| Total | 1166 | Exact saved regression count. |

Additional artifact-level caveats:

- `counts.tsv` counts `7197` PASS result lines, but `current-passes.normalized.txt` contains `7196` unique PASS paths. The duplicate PASS path is `php-src/ext/pdo/tests/pdo_037.phpt`.
- Candidate `current-status.normalized.tsv` has `18940` status/path rows covering `18822` unique PHPT paths; `118` paths have conflicting duplicate statuses.
- Accepted baseline `current-status.normalized.tsv` has `21943` status/path rows covering `21823` unique PHPT paths.
- Comparing unique paths, the candidate is missing `3001` baseline-status paths; `1136` of those were accepted PASS rows and therefore appear as PASS regressions.
- `aggregate-warnings.tsv` says `missing_results	0`, but that only means every shard produced a `results.txt` file. It does not prove every expected PHPT path received a status.

## Artifact Paths

Primary evidence:

- Candidate regression list: `221205Z/regressions-from-latest-published-passes.txt`
- Candidate normalized status: `221205Z/current-status.normalized.tsv`
- Candidate normalized passes: `221205Z/current-passes.normalized.txt`
- Candidate aggregate results: `221205Z/all-results.txt`
- Candidate counts: `221205Z/counts.tsv`
- Candidate public comparable score: `221205Z/public-comparable-score.tsv`
- Candidate pass-regression summary: `221205Z/pass-regression-summary.tsv`
- Candidate shard logs: `221205Z/shard-01` through `221205Z/shard-06`, plus `221205Z/serial-openbasedir`
- Candidate stdout-only abort evidence: `221205Z/shard-03/stdout.log`, `221205Z/shard-04/stdout.log`
- Accepted baseline passes: `135138Z/current-passes.normalized.txt`
- Accepted baseline status: `135138Z/current-status.normalized.tsv`

The candidate `/tmp` run root was no longer present during inspection, so expected shard membership was reconstructed from `/home/claude/php-src-phpt` at php-src pin `f97ff597429a2fe633665a7e02d97c8077f9f90f` and the saved `run_gate.sh` round-robin rule.

## Low-CPU Commands Used

No full PHPT gate was run. Commands were artifact reads, set comparisons, and small Python `sqlite3`/text-parsing scripts.

```sh
sed -n '1,80p' "$CAND/pass-regression-summary.tsv"
sed -n '1,80p' "$CAND/counts.tsv"
sed -n '1,80p' "$CAND/public-comparable-score.tsv"
sed -n '1,40p' "$CAND/regressions-from-latest-published-passes.txt"
sed -n '1,40p' "$CAND/current-status.normalized.tsv"
find "$CAND" -maxdepth 2 -type f | sort
rg -n "ERROR: cannot open directory|Failed to open directory" "$CAND"/shard-*/*.log "$CAND"/serial-openbasedir/*.log
rg -n "rangeformatter_icu63_compatibility|openssl_libctx_without_zts_argon|grep2\\.phpt" "$CAND"/shard-*/stdout.log "$CAND"/shard-*/run-tests.log
python - <<'PY'
from pathlib import Path
base = Path("135138Z/current-passes.normalized.txt")
cand = Path("221205Z/current-passes.normalized.txt")
print(len(set(base.read_text().splitlines()) - set(cand.read_text().splitlines())))
PY
```

The actual Python scripts used absolute paths and also parsed:

- path-level status buckets from `current-status.normalized.tsv`
- duplicate/conflicting candidate result rows from `all-results.txt`
- expected PHPT membership from `/home/claude/php-src-phpt`
- test titles and SKIPIF reasons from shard stdout/run-tests logs where available

## Failure Symptom Clusters

| symptom | count | evidence | representative rows |
| --- | ---: | --- | --- |
| Absent candidate row/status | 1136 | Present in baseline PASS set, absent from candidate PASS set, candidate status, and candidate aggregate results. | `php-src/ext/standard/tests/strings/005.phpt`, `php-src/ext/standard/tests/array/006.phpt`, `php-src/ext/spl/tests/ArrayObject/ArrayObject_clone_other_std_props.phpt`, `php-src/ext/reflection/tests/001.phpt` |
| Concrete `FAILED`: readonly/internal property semantics | 15 | Shard stdout/run-tests titles. | `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt`, `php-src/Zend/tests/property_hooks/gh19548.phpt`, `php-src/ext/bcmath/tests/number/properties_unset.phpt`, `php-src/ext/xmlreader/tests/014.phpt` |
| Concrete `FAILED`: exception/assert/GC/serialize behavior | 6 | Shard stdout/run-tests titles. | `php-src/Zend/tests/assert/expect_008.phpt`, `php-src/Zend/tests/gc/bug63635.phpt`, `php-src/Zend/tests/serialize/bug76502.phpt`, `php-src/Zend/tests/uncaught_exception_error_supression.phpt` |
| Concrete `FAILED`: class lifecycle/iterator legacy rows | 4 | Shard stdout/run-tests titles. | `php-src/tests/classes/ctor_dtor.phpt`, `php-src/tests/classes/destructor_and_echo.phpt`, `php-src/tests/classes/factory_and_singleton_002.phpt`, `php-src/tests/classes/iterators_002.phpt` |
| Concrete `FAILED`: variance/tentative-return inheritance | 1 | Shard stdout/run-tests title. | `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt` |
| Concrete `FAILED`: opcache leak PHPT | 1 | Shard stdout says `SCCP 037: Memory leak`. | `php-src/ext/opcache/tests/opt/sccp_037.phpt` |
| Concrete `BORKED`: SKIPIF missing constants | 3 | Shard stdout says invalid SKIPIF output. | `INTL_ICU_VERSION` in `rangeformatter_icu63_compatibility.phpt`; `ZEND_THREAD_SAFE` in `openssl_libctx_without_zts_argon.phpt`; `PCRE_JIT_SUPPORT` in `grep2.phpt` |

Shard-level harness symptoms:

- All six candidate shards exited `1`, plus serialized open_basedir exited `1`.
- `shard-03/stdout.log` and `shard-04/stdout.log` contain `ERROR: cannot open directory: .../run-tests-harnesses/shard-0{3,4}/ext/pdo/tests`.
- Candidate shards 03 and 04 have `results.txt` plus stdout/stderr, but no `run-tests.log`.
- Reconstructed expected membership shows missing candidate status paths in all six shards, not only shards 03/04. The missing regression rows by reconstructed expected shard are: shard-04 `307`, shard-05 `297`, shard-03 `199`, shard-06 `188`, shard-01 `74`, shard-02 `71`.

## Extension Clusters

| cluster | total | absent | failed | borked | examples |
| --- | ---: | ---: | ---: | ---: | --- |
| `ext/standard` | 794 | 792 | 2 | 0 | `php-src/ext/standard/tests/array/006.phpt`, `php-src/ext/standard/tests/array/007.phpt`, `php-src/ext/standard/tests/array/array_change_key_case.phpt` |
| `ext/spl` | 137 | 137 | 0 | 0 | `php-src/ext/spl/tests/ArrayObject/ArrayObject_clone_other_std_props.phpt`, `php-src/ext/spl/tests/ArrayObject/ArrayObject_modify_shared_object_properties.phpt`, `php-src/ext/spl/tests/ArrayObject/ArrayObject_proptable_canonicalization.phpt` |
| `ext/reflection` | 110 | 110 | 0 | 0 | `php-src/ext/reflection/tests/001.phpt`, `php-src/ext/reflection/tests/007.phpt`, `php-src/ext/reflection/tests/013.phpt` |
| `ext/uri` | 41 | 41 | 0 | 0 | `php-src/ext/uri/tests/gh19979.phpt`, `php-src/ext/uri/tests/rfc3986/equivalence/equals_false_host.phpt`, `php-src/ext/uri/tests/rfc3986/equivalence/equals_false_scheme.phpt` |
| `ext/posix` | 16 | 16 | 0 | 0 | `php-src/ext/posix/tests/001.phpt`, `php-src/ext/posix/tests/bug75696.phpt`, `php-src/ext/posix/tests/posix_errno_basic.phpt` |
| `Zend/tests` | 15 | 0 | 15 | 0 | `php-src/Zend/tests/assert/expect_008.phpt`, `php-src/Zend/tests/assert/expect_011.phpt`, `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt` |
| `ext/tokenizer` | 14 | 14 | 0 | 0 | `php-src/ext/tokenizer/tests/003.phpt`, `php-src/ext/tokenizer/tests/PhpToken_final_constructor.phpt`, `php-src/ext/tokenizer/tests/PhpToken_methods.phpt` |
| `ext/xmlreader` | 9 | 8 | 1 | 0 | `php-src/ext/xmlreader/tests/001.phpt`, `php-src/ext/xmlreader/tests/003.phpt`, `php-src/ext/xmlreader/tests/004.phpt` |
| `ext/session` | 7 | 7 | 0 | 0 | `php-src/ext/session/tests/006.phpt`, `php-src/ext/session/tests/009.phpt`, `php-src/ext/session/tests/bug24592.phpt` |
| `ext/random` | 4 | 4 | 0 | 0 | `php-src/ext/random/tests/01_functions/lcg_value_basic.phpt`, `php-src/ext/random/tests/01_functions/rand_basic.phpt`, `php-src/ext/random/tests/01_functions/rand_inverted_order.phpt` |
| `tests/classes` | 4 | 0 | 4 | 0 | `php-src/tests/classes/ctor_dtor.phpt`, `php-src/tests/classes/destructor_and_echo.phpt`, `php-src/tests/classes/factory_and_singleton_002.phpt` |
| `sapi` | 3 | 3 | 0 | 0 | `php-src/sapi/cli/tests/002.phpt`, `php-src/sapi/cli/tests/021.phpt`, `php-src/sapi/cli/tests/bug70006.phpt` |
| `ext/bcmath` | 2 | 0 | 2 | 0 | `php-src/ext/bcmath/tests/number/properties_unset.phpt`, `php-src/ext/bcmath/tests/number/properties_write_error.phpt` |
| `ext/date` | 2 | 0 | 2 | 0 | `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`, `php-src/ext/date/tests/DatePeriod_properties2.phpt` |
| `ext/intl` | 1 | 0 | 0 | 1 | `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` |
| `ext/opcache` | 1 | 0 | 1 | 0 | `php-src/ext/opcache/tests/opt/sccp_037.phpt` |
| `ext/openssl` | 1 | 0 | 0 | 1 | `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt` |
| `ext/pcre` | 1 | 0 | 0 | 1 | `php-src/ext/pcre/tests/grep2.phpt` |
| `ext/phar` | 1 | 1 | 0 | 0 | `php-src/ext/phar/tests/bug79797.phpt` |
| `ext/sodium` | 1 | 1 | 0 | 0 | `php-src/ext/sodium/tests/installed.phpt` |
| `ext/zip` | 1 | 1 | 0 | 0 | `php-src/ext/zip/tests/001.phpt` |
| `ext/zlib` | 1 | 1 | 0 | 0 | `php-src/ext/zlib/tests/ob_002.phpt` |

## Top Test Directory Clusters

| test directory | regressions | dominant symptom | examples |
| --- | ---: | --- | --- |
| `php-src/ext/standard/tests/strings` | 197 | ABSENT (197) | `005.phpt`, `006.phpt`, `addcslashes_001.phpt`, `addcslashes_002.phpt` |
| `php-src/ext/standard/tests/array` | 175 | ABSENT (175) | `006.phpt`, `007.phpt`, `array_change_key_case.phpt`, `array_change_key_case_flag_error.phpt` |
| `php-src/ext/standard/tests/file` | 160 | ABSENT (160) | `005_error.phpt`, `005_variation2.phpt`, `006_variation1.phpt`, `007_variation12.phpt` |
| `php-src/ext/reflection/tests` | 104 | ABSENT (104) | `001.phpt`, `007.phpt`, `013.phpt`, `014.phpt` |
| `php-src/ext/spl/tests` | 73 | ABSENT (73) | `DirectoryIterator_getBasename_basic_test.phpt`, `DirectoryIterator_getExtension_basic.phpt`, `DirectoryIterator_uninitialized.phpt`, `SplDoublyLinkedList_bottom_empty.phpt` |
| `php-src/ext/standard/tests/math` | 53 | ABSENT (53) | `acos_basic.phpt`, `acos_basiclong_64bit.phpt`, `asin_basic.phpt`, `asin_basiclong_64bit.phpt` |
| `php-src/ext/standard/tests/array/sort` | 49 | ABSENT (49) | `array_multisort_basic1.phpt`, `array_multisort_natural_case.phpt`, `array_multisort_natural_incase.phpt`, `array_multisort_variation1.phpt` |
| `php-src/ext/standard/tests/general_functions` | 44 | ABSENT (44) | `001.phpt`, `002.phpt`, `008.phpt`, `009.phpt` |
| `php-src/ext/spl/tests/ArrayObject` | 29 | ABSENT (29) | `ArrayObject_clone_other_std_props.phpt`, `ArrayObject_modify_shared_object_properties.phpt`, `ArrayObject_proptable_canonicalization.phpt`, `ArrayObject_std_props_no_recursion.phpt` |
| `php-src/ext/spl/tests/SplFileObject` | 19 | ABSENT (19) | `SplFileObject_fgetcsv_basic.phpt`, `SplFileObject_fgetcsv_delimiter_basic.phpt`, `SplFileObject_fgetcsv_escape_default.phpt`, `SplFileObject_fputcsv_variation1.phpt` |
| `php-src/ext/posix/tests` | 16 | ABSENT (16) | `001.phpt`, `bug75696.phpt`, `posix_errno_basic.phpt`, `posix_errno_variation1.phpt` |
| `php-src/ext/standard/tests/array/array_walk` | 14 | ABSENT (14) | `array_walk_basic1.phpt`, `array_walk_object2.phpt`, `array_walk_objects.phpt`, `array_walk_recursive_basic2.phpt` |
| `php-src/ext/standard/tests/dir` | 14 | ABSENT (14) | `bug71542.phpt`, `chdir_error2.phpt`, `closedir_variation2.phpt`, `dir_variation6.phpt` |
| `php-src/ext/standard/tests/serialize` | 14 | ABSENT (14) | `002.phpt`, `003.phpt`, `bug23298.phpt`, `bug30234.phpt` |
| `php-src/ext/tokenizer/tests` | 14 | ABSENT (14) | `003.phpt`, `PhpToken_final_constructor.phpt`, `PhpToken_methods.phpt`, `attributes.phpt` |
| `php-src/ext/uri/tests/rfc3986/parsing` | 14 | ABSENT (14) | `basic_error_null_byte.phpt`, `basic_success_all.phpt`, `host_error_multibyte.phpt`, `host_error_reserved.phpt` |
| `php-src/ext/standard/tests/class_object` | 12 | ABSENT (12) | `get_class_methods_variation_001.phpt`, `get_class_methods_variation_002.phpt`, `get_declared_classes_variation1.phpt`, `get_declared_interfaces_basic_001.phpt` |
| `php-src/ext/uri/tests/whatwg/parsing` | 12 | ABSENT (12) | `basic_sucess_urn.phpt`, `host_error_empty1.phpt`, `host_error_null_byte.phpt`, `host_success_empty4.phpt` |
| `php-src/ext/standard/tests/streams` | 10 | ABSENT (10) | `bug75031.phpt`, `gh19570.phpt`, `stream_context_get_params_001.phpt`, `stream_context_set_option_basic.phpt` |
| `php-src/ext/standard/tests/url` | 10 | ABSENT (10) | `base64_encode_basic_001.phpt`, `base64_encode_basic_002.phpt`, `bug54180.phpt`, `bug55273.phpt` |
| `php-src/ext/spl/tests/SplObjectStorage` | 9 | ABSENT (9) | `SplObjectStorage_current_empty_storage.phpt`, `SplObjectStorage_removeAllExcept_basic.phpt`, `SplObjectStorage_seek.phpt`, `SplObjectStorage_unserialize_reference.phpt` |
| `php-src/ext/xmlreader/tests` | 9 | ABSENT (8) | `001.phpt`, `003.phpt`, `004.phpt`, `009.phpt` |
| `php-src/ext/session/tests` | 7 | ABSENT (7) | `006.phpt`, `009.phpt`, `bug24592.phpt`, `session_cache_limiter_basic.phpt` |
| `php-src/ext/spl/tests/autoloading` | 7 | ABSENT (7) | `bug61697.phpt`, `bug74372.phpt`, `spl_autoload_003.phpt`, `spl_autoload_009.phpt` |
| `php-src/ext/standard/tests/assert` | 6 | ABSENT (6) | `assert_basic2.phpt`, `assert_basic3.phpt`, `assert_closures.phpt`, `assert_closures_multiple.phpt` |
| `php-src/ext/standard/tests/directory` | 6 | ABSENT (4), FAILED (2) | `DirectoryClass_cannot_construct.phpt`, `DirectoryClass_cannot_serialize.phpt`, `DirectoryClass_readonly_handle.phpt`, `DirectoryClass_readonly_path.phpt` |
| `php-src/tests/classes` | 4 | FAILED (4) | `ctor_dtor.phpt`, `destructor_and_echo.phpt`, `factory_and_singleton_002.phpt`, `iterators_002.phpt` |
| `php-src/Zend/tests` and subdirectories | 15 | FAILED (15) | `expect_008.phpt`, `property_readonly_001.phpt`, `gh19548.phpt`, `bug76502.phpt` |

Remaining nonzero directories are singletons or pairs: `ext/bcmath/tests/number`, `ext/date/tests`, `ext/intl/tests/rangeformatter`, `ext/opcache/tests/opt`, `ext/openssl/tests`, `ext/pcre/tests`, `ext/phar/tests`, `ext/sodium/tests`, `ext/zip/tests`, `ext/zlib/tests`, `sapi/cli/tests`, and small URI/standard subdirectories already represented by the extension table.

## Concrete Non-PASS Rows

### FAILED

- `php-src/Zend/tests/assert/expect_008.phpt`
- `php-src/Zend/tests/assert/expect_011.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt`
- `php-src/Zend/tests/attributes/deprecated/property_readonly_002.phpt`
- `php-src/Zend/tests/attributes/override/properties_08.phpt`
- `php-src/Zend/tests/bug73989.phpt`
- `php-src/Zend/tests/gc/bug63635.phpt`
- `php-src/Zend/tests/property_hooks/gh19548.phpt`
- `php-src/Zend/tests/property_hooks/gh19548_002.phpt`
- `php-src/Zend/tests/readonly_classes/readonly_class_property1.phpt`
- `php-src/Zend/tests/readonly_classes/readonly_class_property2.phpt`
- `php-src/Zend/tests/readonly_props/readonly_trait_mismatch.phpt`
- `php-src/Zend/tests/serialize/bug76502.phpt`
- `php-src/Zend/tests/type_declarations/variance/internal_parent/unresolvable_inheritance_check_param.phpt`
- `php-src/Zend/tests/uncaught_exception_error_supression.phpt`
- `php-src/ext/bcmath/tests/number/properties_unset.phpt`
- `php-src/ext/bcmath/tests/number/properties_write_error.phpt`
- `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`
- `php-src/ext/date/tests/DatePeriod_properties2.phpt`
- `php-src/ext/opcache/tests/opt/sccp_037.phpt`
- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`
- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt`
- `php-src/ext/xmlreader/tests/014.phpt`
- `php-src/tests/classes/ctor_dtor.phpt`
- `php-src/tests/classes/destructor_and_echo.phpt`
- `php-src/tests/classes/factory_and_singleton_002.phpt`
- `php-src/tests/classes/iterators_002.phpt`

### BORKED

- `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt`: SKIPIF fatal for undefined `INTL_ICU_VERSION`.
- `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt`: SKIPIF fatal for undefined `ZEND_THREAD_SAFE`.
- `php-src/ext/pcre/tests/grep2.phpt`: SKIPIF fatal for undefined `PCRE_JIT_SUPPORT`.

## First Repair Lane Recommendations

1. **Gate completeness/accounting lane**
   - Fix the gate aggregator to compare the expected PHPT path set against normalized current-status paths and block as incomplete when rows are absent.
   - The current `missing_results=0` check is insufficient because all shard files can exist while thousands of expected rows have no status.
   - Representative rows: `php-src/ext/standard/tests/strings/005.phpt`, `php-src/ext/standard/tests/array/006.phpt`, `php-src/ext/spl/tests/ArrayObject/ArrayObject_clone_other_std_props.phpt`, `php-src/ext/reflection/tests/001.phpt`.

2. **PHPT run-tests harness directory-layout lane**
   - Investigate and fix `run-tests-harnesses/shard-03/ext/pdo/tests` and `run-tests-harnesses/shard-04/ext/pdo/tests` missing-directory aborts.
   - Preserve shard test lists in evidence so future audits do not need to reconstruct expected membership from php-src.
   - Representative rows behind missing-status clusters: `php-src/ext/posix/tests/001.phpt`, `php-src/ext/random/tests/01_functions/rand_basic.phpt`, `php-src/ext/reflection/tests/014.phpt`, `php-src/ext/standard/tests/file/005_error.phpt`.

3. **SKIPIF constant/environment lane**
   - Repair wrapper/runtime exposure of constants needed by PHPT SKIPIF scripts, or make the PHPT wrapper delegate SKIPIF probes to a compatible system PHP where appropriate.
   - Representative rows: `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt`, `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt`, `php-src/ext/pcre/tests/grep2.phpt`.

4. **Readonly/internal property semantics lane**
   - Focus on readonly/internal object property write/unset diagnostics and property hook interactions.
   - Representative rows: `php-src/Zend/tests/attributes/deprecated/property_readonly_001.phpt`, `php-src/Zend/tests/property_hooks/gh19548.phpt`, `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`, `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`, `php-src/ext/xmlreader/tests/014.phpt`.

5. **Core lifecycle/exception/iterator behavior lane**
   - Focus on destructor ordering, uncaught exception behavior under suppression/assertion paths, serialize error chains, and legacy iterator/class lifecycle rows.
   - Representative rows: `php-src/tests/classes/ctor_dtor.phpt`, `php-src/tests/classes/destructor_and_echo.phpt`, `php-src/tests/classes/iterators_002.phpt`, `php-src/Zend/tests/assert/expect_008.phpt`, `php-src/Zend/tests/serialize/bug76502.phpt`.

The opcache row `php-src/ext/opcache/tests/opt/sccp_037.phpt` is a separate one-row memory-leak PHPT and should not be prioritized before the five lanes above unless a focused replay proves it is hiding a broader leak cluster.
