# 221205Z Secondary Extension Regression Shard

Lane: 20, developer-83

Scope: read-only M0 shard for `php-src/ext/{uri,tokenizer,xmlreader,session,posix,random}` plus one-off `bcmath`, `date`, `intl`, `opcache`, `openssl`, `pcre`, `phar`, `sodium`, `zip`, and `zlib` regressions from the blocked `221205Z` public PHPT gate. No compiler/runtime source edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate result/status artifacts:
  `all-results.txt`, `current-status.normalized.tsv`, `shard-*/results.txt`,
  `shard-*/stdout.log`, `shard-*/run-tests.log`, `aggregate-warnings.tsv`,
  `shard-exit-codes.tsv`
- PHPT source checkout for title/section inspection:
  `/home/claude/php-src-phpt`

The overall blocked gate remains `7197 / 20294` by raw public score, with `1166` latest-public PASS regressions in the normalized pass-set summary. This shard accounts for `103` of those regression rows.

## Counts

| Extension | Regression rows |
| --- | ---: |
| `uri` | 41 |
| `posix` | 16 |
| `tokenizer` | 14 |
| `xmlreader` | 9 |
| `session` | 7 |
| `random` | 4 |
| `bcmath` | 2 |
| `date` | 2 |
| `intl` | 1 |
| `opcache` | 1 |
| `openssl` | 1 |
| `pcre` | 1 |
| `phar` | 1 |
| `sodium` | 1 |
| `zip` | 1 |
| `zlib` | 1 |
| **Total** | **103** |

Status coverage for these `103` rows after normalizing `all-results.txt` absolute paths back to `php-src/...`:

| Candidate artifact status | Count |
| --- | ---: |
| Absent from `current-status.normalized.tsv` and normalized `all-results.txt` | 94 |
| `FAILED` | 6 |
| `BORKED` | 3 |
| **Total** | **103** |

Per-extension status split:

| Extension | Absent | FAILED | BORKED |
| --- | ---: | ---: | ---: |
| `uri` | 41 | 0 | 0 |
| `posix` | 16 | 0 | 0 |
| `tokenizer` | 14 | 0 | 0 |
| `xmlreader` | 8 | 1 | 0 |
| `session` | 7 | 0 | 0 |
| `random` | 4 | 0 | 0 |
| `bcmath` | 0 | 2 | 0 |
| `date` | 0 | 2 | 0 |
| `intl` | 0 | 0 | 1 |
| `opcache` | 0 | 1 | 0 |
| `openssl` | 0 | 0 | 1 |
| `pcre` | 0 | 0 | 1 |
| `phar` | 1 | 0 | 0 |
| `sodium` | 1 | 0 | 0 |
| `zip` | 1 | 0 | 0 |
| `zlib` | 1 | 0 | 0 |

`aggregate-warnings.tsv` says `missing_results	0`, and all six shard exit rows are `1`. That means the artifact set does not label the 94 absent regression rows as missing expected results, even though they are in `regressions-from-latest-published-passes.txt` and not present in the candidate status/result views. Treat those 94 rows as replay/evidence-integrity targets before assigning compiler semantic root causes.

## Direct Status Rows

Rows with preserved direct candidate status:

| Status | Row | Preserved symptom |
| --- | --- | --- |
| `FAILED` | `php-src/ext/bcmath/tests/number/properties_unset.phpt` | Shard 03 preserved `results.txt`/`stdout.log` only; stdout records the failure. PHPT expects readonly-property unset diagnostics for `BcMath\Number::$value` and `$scale`. |
| `FAILED` | `php-src/ext/bcmath/tests/number/properties_write_error.phpt` | Shard 04 preserved `results.txt`/`stdout.log` only; stdout records the failure. PHPT expects readonly-property write diagnostics for `BcMath\Number::$value` and `$scale`. |
| `FAILED` | `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt` | `run-tests.log` diff shows expected `Cannot modify readonly property DatePeriod::$interval`, but actual output contains `Cannot modify protected(set) readonly property DatePeriod::$interval from global scope` variants. |
| `FAILED` | `php-src/ext/date/tests/DatePeriod_properties2.phpt` | `run-tests.log` diff shows `DatePeriod` readonly-property diagnostics changed to `protected(set) readonly property ... from global scope` variants. |
| `FAILED` | `php-src/ext/opcache/tests/opt/sccp_037.phpt` | Shard 04 preserved `results.txt`/`stdout.log` only; stdout records `SCCP 037: Memory leak` as failed. PHPT body is `[!![[new ERROR]]];` with expected `DONE`. |
| `FAILED` | `php-src/ext/xmlreader/tests/014.phpt` | `run-tests.log` diff shows expected `Cannot modify readonly property XMLReader::$value/$name`, but actual output contains `Cannot modify protected(set) readonly property ... from global scope`. |
| `BORKED` | `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` | Invalid `SKIPIF`: fatal undefined constant `INTL_ICU_VERSION` in the generated `.skip.php`. |
| `BORKED` | `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt` | Invalid `SKIPIF`: fatal undefined constant `ZEND_THREAD_SAFE` in the generated `.skip.php`. |
| `BORKED` | `php-src/ext/pcre/tests/grep2.phpt` | Invalid `SKIPIF`: fatal undefined constant `PCRE_JIT_SUPPORT` in the generated `.skip.php`. |

The direct failures split into two small mechanisms:

- Readonly-property diagnostic drift for `DatePeriod`, `XMLReader`, and likely the `BcMath\Number` property rows. The preserved diffs prove this for `date`/`xmlreader`; the `bcmath` shards did not preserve `run-tests.log` or diff payloads, so replay is needed before making the same claim there.
- Extension metadata/constant exposure in `SKIPIF` for `intl`, `openssl`, and `pcre`. These BORKED rows are not body-execution failures; they fail before the test body because skip scripts reference missing extension constants.

## Absent-Result Clusters

The 94 absent rows are concentrated in feature clusters that should be replayed as a control-plane/artifact problem first:

| Cluster | Rows | Representative rows |
| --- | ---: | --- |
| URI RFC3986/WHATWG parsing, getters, equivalence | 41 | `php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt`, `php-src/ext/uri/tests/whatwg/equivalence/equals_true_normalization2.phpt` |
| POSIX functions and errno/user/group/session IDs | 16 | `php-src/ext/posix/tests/posix_uname_basic.phpt`, `php-src/ext/posix/tests/posix_getpwuid_basic.phpt` |
| Tokenizer and `PhpToken` APIs | 14 | `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt`, `php-src/ext/tokenizer/tests/PhpToken_methods.phpt` |
| XMLReader baseline/virtual property rows excluding `014.phpt` | 8 | `php-src/ext/xmlreader/tests/virtual_properties2.phpt`, `php-src/ext/xmlreader/tests/readString_basic.phpt` |
| Session startup/cache/write-close behavior | 7 | `php-src/ext/session/tests/session_cache_limiter_basic.phpt`, `php-src/ext/session/tests/session_write_close_variation4.phpt` |
| Random legacy functions and reflection | 4 | `php-src/ext/random/tests/01_functions/rand_basic.phpt`, `php-src/ext/random/tests/01_functions/reflection.phpt` |
| One-off extension presence/edge rows | 4 | `php-src/ext/phar/tests/bug79797.phpt`, `php-src/ext/sodium/tests/installed.phpt`, `php-src/ext/zip/tests/001.phpt`, `php-src/ext/zlib/tests/ob_002.phpt` |

Representative PHPT title/section inspection:

| Row | PHPT title | Sections |
| --- | --- | --- |
| `php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt` | `Test Uri\Rfc3986\Uri parsing - basic - all components` | `TEST`, `FILE`, `EXPECTF` |
| `php-src/ext/posix/tests/posix_uname_basic.phpt` | `Test posix_uname() function : basic functionality` | `TEST`, `EXTENSIONS`, `FILE`, `EXPECTF` |
| `php-src/ext/random/tests/01_functions/rand_basic.phpt` | `Test  rand() - basic function test rand()` | `TEST`, `FILE`, `EXPECTF` |
| `php-src/ext/session/tests/session_cache_limiter_basic.phpt` | `Test session_cache_limiter() function : basic functionality` | `TEST`, `EXTENSIONS`, `SKIPIF`, `FILE`, `EXPECT` |
| `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt` | `Reconstructing a script using token_get_all()` | `TEST`, `EXTENSIONS`, `FILE`, `EXPECT` |
| `php-src/ext/xmlreader/tests/virtual_properties2.phpt` | `Virtual property existence tests` | `TEST`, `EXTENSIONS`, `FILE`, `EXPECT` |

## Recommended Replay Set

Start with two replay groups rather than one mixed implementation lane:

1. Absent-result/control-plane sample:
   `php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt`,
   `php-src/ext/tokenizer/tests/token_get_all_variation19.phpt`,
   `php-src/ext/posix/tests/posix_uname_basic.phpt`,
   `php-src/ext/session/tests/session_cache_limiter_basic.phpt`,
   `php-src/ext/random/tests/01_functions/rand_basic.phpt`,
   `php-src/ext/xmlreader/tests/virtual_properties2.phpt`.
2. Direct-status sample:
   `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt`,
   `php-src/ext/xmlreader/tests/014.phpt`,
   `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt`,
   `php-src/ext/openssl/tests/openssl_libctx_without_zts_argon.phpt`,
   `php-src/ext/pcre/tests/grep2.phpt`,
   plus one of the `bcmath` property rows after confirming whether diff payloads are available in a fresh replay.

If the absent-result group reproduces as missing from candidate status/result normalization again, keep it in the M0 control-plane lane. If direct replay confirms the readonly diagnostic drift and `SKIPIF` constant errors, those can become narrow fix lanes without mixing them with the 94 absent-result rows.

## Commands Run

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
regs = (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines()
target_ext = {'uri','tokenizer','xmlreader','session','posix','random','bcmath','date','intl','opcache','openssl','pcre','phar','sodium','zip','zlib'}
rows = [r for r in regs if r.startswith('php-src/ext/') and len(r.split('/')) >= 3 and r.split('/')[2] in target_ext]
print(len(rows))
print(Counter(r.split('/')[2] for r in rows))
PY
```

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
# Build normalized current-status and all-results maps, then compare target row statuses.
PY
```

```sh
python3 - <<'PY'
from pathlib import Path
base = Path('/home/claude/php-src-phpt')
# Inspect --TEST-- titles and section names for representative target PHPTs.
PY
```

```sh
find /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-03 \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-04 \
  -maxdepth 1 -type f -print
```

## Next Action

Do not start a broad extension implementation lane from this report alone. First replay the absent-result sample to determine whether the 94-row cluster is a result-normalization/control-plane failure. Separately replay the nine direct-status rows and split confirmed fixes into readonly-property diagnostic parity and `SKIPIF` extension-constant availability.
