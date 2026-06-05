# Focused Replay: Secondary Extension Rows (developer-112 replacement)

Lane 85 read-only report for the blocked candidate gate
`phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`.
This narrows the completed `221205Z-secondary-ext.md` shard evidence to eight
representative latest-public PASS regressions and classifies whether the
candidate evidence shows semantic failure, control-plane absence, or replay
unavailability.

No compiler/runtime source files were edited. No full PHPT gate was run.
`DEVELOPMENT.md` was requested by the lane instructions but is absent under
`/home/claude/php-to-native-compiler`.

## Inputs

- Completed secondary extension shard report:
  `/home/claude/php-to-native-compiler/.harness/reports/221205Z-secondary-ext.md`
- Accepted baseline artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Blocked candidate artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Accepted public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Candidate public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- php-src checkout: `/home/claude/php-src-phpt`
- php-src pin verified in that checkout:
  `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Wrapper: `/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`

Gate score context:

```text
accepted public-comparable-score.tsv: passed=7873 pinned_runnable=20294 public_percent=38.79
candidate public-comparable-score.tsv: passed=7197 pinned_runnable=20294 public_percent=35.46
candidate pass-regression-summary.tsv: baseline_passes=7869 current_passes=7196 pass_regressions=1166
```

## Shard Cross-Check

The completed secondary extension shard owns 103 latest-public PASS
regressions:

```text
owned_rows 103
by_ext {'bcmath': 2, 'date': 2, 'intl': 1, 'opcache': 1, 'openssl': 1, 'pcre': 1, 'phar': 1, 'posix': 16, 'random': 4, 'session': 7, 'sodium': 1, 'tokenizer': 14, 'uri': 41, 'xmlreader': 9, 'zip': 1, 'zlib': 1}
status_counts {'BORKED': 3, 'FAILED': 6, 'MISSING': 94}
```

Shard-level control-plane evidence remains relevant:

```text
shard-03/stdout.log: ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests
shard-04/stdout.log: ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests
shard-01, shard-02, shard-05, shard-06 reached "Report saved to:"
```

## Representative Rows

These eight rows cover the assigned secondary-extension areas while avoiding
eval and variable-variable PHPT bodies. The tokenizer row from the earlier
shard recommendation, `token_get_all_variation19.phpt`, contains an eval marker
and was replaced with `PhpToken_methods.phpt`.

| Row | PHPT title | Bucket |
| --- | --- | --- |
| `php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt` | `Test Uri\Rfc3986\Uri parsing - basic - all components` | URI absent cluster |
| `php-src/ext/tokenizer/tests/PhpToken_methods.phpt` | `PhpToken instance methods` | tokenizer/PhpToken absent cluster |
| `php-src/ext/posix/tests/posix_uname_basic.phpt` | `Test posix_uname() function : basic functionality` | POSIX absent cluster |
| `php-src/ext/session/tests/session_cache_limiter_basic.phpt` | `Test session_cache_limiter() function : basic functionality` | session absent cluster |
| `php-src/ext/random/tests/01_functions/rand_basic.phpt` | `Test  rand() - basic function test rand()` | random absent cluster |
| `php-src/ext/xmlreader/tests/014.phpt` | `XMLReader: libxml2 XML Reader, read-only element values cannot be modified` | direct FAILED readonly diagnostic |
| `php-src/ext/bcmath/tests/number/properties_unset.phpt` | `BcMath\Number properties unset` | direct FAILED readonly property row |
| `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` | `Test IntlNumberRangeFormatter::createFromSkeleton throws error for ICU < 63` | direct BORKED SKIPIF constant |

Source scan found no eval or variable-variable markers in these eight selected
rows.

## Artifact Status Join

Command shape:

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter
ACC=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67')
CAND=Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
PHP_SRC=Path('/home/claude/php-src-phpt')
# Load accepted/candidate current-status.normalized.tsv and all-results.txt,
# normalize absolute /php-src/ result paths, and join the eight rows below.
PY
```

Result:

```text
row	title	sections	late_markers	in_regression_list	accepted_status	accepted_all_results	candidate_status	candidate_all_results
php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt	Test Uri\Rfc3986\Uri parsing - basic - all components	TEST,FILE,EXPECTF	none	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/tokenizer/tests/PhpToken_methods.phpt	PhpToken instance methods	TEST,EXTENSIONS,FILE,EXPECT	none	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/posix/tests/posix_uname_basic.phpt	Test posix_uname() function : basic functionality	TEST,EXTENSIONS,FILE,EXPECTF	none	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/session/tests/session_cache_limiter_basic.phpt	Test session_cache_limiter() function : basic functionality	TEST,EXTENSIONS,SKIPIF,FILE,EXPECT	none	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/random/tests/01_functions/rand_basic.phpt	Test  rand() - basic function test rand()	TEST,FILE,EXPECTF	none	True	PASSED	PASSED	MISSING	MISSING
php-src/ext/xmlreader/tests/014.phpt	XMLReader: libxml2 XML Reader, read-only element values cannot be modified	TEST,CREDITS,EXTENSIONS,FILE,CLEAN,EXPECT	none	True	PASSED	PASSED	FAILED	FAILED
php-src/ext/bcmath/tests/number/properties_unset.phpt	BcMath\Number properties unset	TEST,EXTENSIONS,FILE,EXPECT	none	True	PASSED	PASSED	FAILED	FAILED
php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt	Test IntlNumberRangeFormatter::createFromSkeleton throws error for ICU < 63	TEST,EXTENSIONS,SKIPIF,FILE,EXPECT	none	True	PASSED	PASSED	BORKED	BORKED

counts
accepted_status {'PASSED': 8}
accepted_all_results {'PASSED': 8}
candidate_status {'BORKED': 1, 'FAILED': 2, 'MISSING': 5}
candidate_all_results {'BORKED': 1, 'FAILED': 2, 'MISSING': 5}
```

## Focused Replay Availability

The documented focused replay shape requires historical accepted and candidate
release `phpc` binaries through `PHPC_BIN`. The wrapper and pinned php-src
checkout are present, but both historical binaries are absent:

```text
missing	/tmp/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/cargo-target/release/phpc
missing	/tmp/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/cargo-target/release/phpc
present executable	/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper
present executable	/home/claude/php-src-phpt/run-tests.php
```

Focused `run-tests.php` replay was therefore not executed. Running those rows
without the historical binaries would measure a broken replay setup rather than
accepted-vs-candidate compiler behavior. Rebuilding historical release binaries
was not done in this M0 read-only report lane.

The row-list paths prepared for replay would be:

```text
/home/claude/php-src-phpt/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt
/home/claude/php-src-phpt/ext/tokenizer/tests/PhpToken_methods.phpt
/home/claude/php-src-phpt/ext/posix/tests/posix_uname_basic.phpt
/home/claude/php-src-phpt/ext/session/tests/session_cache_limiter_basic.phpt
/home/claude/php-src-phpt/ext/random/tests/01_functions/rand_basic.phpt
/home/claude/php-src-phpt/ext/xmlreader/tests/014.phpt
/home/claude/php-src-phpt/ext/bcmath/tests/number/properties_unset.phpt
/home/claude/php-src-phpt/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt
```

## Candidate Failure Evidence

The five `MISSING` rows are in the accepted PASS baseline and candidate
regression list, but absent from both candidate normalized status and aggregate
results. There is no row-level candidate diff for them, so they should remain
control-plane/result-coverage symptoms until focused replay with restored
binaries proves semantic failure.

The two explicit `FAILED` rows have different evidence strength:

- `php-src/ext/xmlreader/tests/014.phpt`: candidate `shard-02/run-tests.log`
  includes a row-level diff. The expected lines are
  `Cannot modify readonly property XMLReader::$value/$name`, while actual
  output is `Cannot modify protected(set) readonly property ... from global
  scope`. This is a semantic diagnostic parity failure.
- `php-src/ext/bcmath/tests/number/properties_unset.phpt`: candidate
  `shard-03/results.txt` and `stdout.log` preserve the direct `FAILED` status,
  but shard 03 aborted and did not preserve `run-tests.log`. Treat this as an
  explicit candidate failure with likely readonly-property diagnostic/metadata
  relevance, but require focused replay or a fresh row-level diff before
  assigning an implementation root cause.

The direct `BORKED` row is not a test-body semantic failure:

- `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt`:
  candidate `shard-06/stdout.log` and `run-tests.log` show invalid `SKIPIF`
  output caused by fatal undefined constant `INTL_ICU_VERSION` in the generated
  skip script.

## Classification

| Row | Classification | Reason |
| --- | --- | --- |
| `php-src/ext/uri/tests/rfc3986/parsing/basic_success_all.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; no candidate row diff exists; focused replay cannot run without historical `PHPC_BIN`. |
| `php-src/ext/tokenizer/tests/PhpToken_methods.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; source scan avoids eval/variable-variable; no candidate row diff exists. |
| `php-src/ext/posix/tests/posix_uname_basic.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; no evidence that POSIX semantics regressed for this row. |
| `php-src/ext/session/tests/session_cache_limiter_basic.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; no row-level session diff exists. |
| `php-src/ext/random/tests/01_functions/rand_basic.phpt` | control-plane absent; replay unavailable | Accepted PASS; candidate status/result are missing; no row-level random-function diff exists. |
| `php-src/ext/xmlreader/tests/014.phpt` | semantic failure; replay unavailable | Accepted PASS; candidate explicit FAIL; shard diff shows readonly-property diagnostic drift. |
| `php-src/ext/bcmath/tests/number/properties_unset.phpt` | explicit candidate failure; row-level diff unavailable | Accepted PASS; candidate explicit FAIL; shard 03 preserved stdout/results only and aborted before `run-tests.log` was saved. |
| `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` | SKIPIF/environment constant BORKED; replay unavailable | Accepted PASS; candidate explicit BORKED; skip script fatals on undefined `INTL_ICU_VERSION` before the PHPT body. |

## Conclusion

For this focused secondary-extension sample, the blocked 221205Z candidate
shows accepted `PASS=8` and candidate `MISSING=5, FAILED=2, BORKED=1`.

The deterministic next action is split:

- Keep the five absent rows in the harness/control-plane row-coverage bucket
  until historical binaries are restored or rebuilt and a focused replay can
  produce row-level output.
- Treat `xmlreader/014.phpt` as a narrow readonly-property diagnostic parity
  repair candidate.
- Treat `bcmath/properties_unset.phpt` as an explicit failed row needing fresh
  row-level diff before implementation ownership.
- Treat the `intl` row as extension constant/SKIPIF availability work, not as
  body execution semantics.
