# 221205Z standard strings regression shard

Diagnostic/control-plane report for lane 15. This report only inspects
artifacts from the 2026-06-04T22:12:05Z candidate gate and the accepted
2026-06-04T13:51:38Z baseline. It makes no compiler, runtime, source, or
test-list edits and cannot move the public PHPT score by itself.

## Scope and exact count

- Candidate artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Accepted baseline artifact root:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67`
- Candidate public/source head: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- Accepted public/source head: `0b917f67a37d9ca9779d77f87173b628431c2425`
- php-src pin in both runs: `f97ff597429a2fe633665a7e02d97c8077f9f90f`
- Total pass regressions in
  `regressions-from-latest-published-passes.txt`: `1166`
- Exact `php-src/ext/standard/tests/strings/` pass regressions in that file:
  `197`

All 197 assigned rows were observed as `PASSED` in the accepted baseline
`all-results.txt` and `current-status.normalized.tsv`.

## Candidate status breakdown

For the 197 assigned rows:

| Source | PASS | FAIL | SKIP | BORK | Missing/not captured |
| --- | ---: | ---: | ---: | ---: | ---: |
| accepted `all-results.txt` | 197 | 0 | 0 | 0 | 0 |
| accepted `current-status.normalized.tsv` | 197 | 0 | 0 | 0 | 0 |
| candidate `all-results.txt` | 0 | 0 | 0 | 0 | 197 |
| candidate `current-status.normalized.tsv` | 0 | 0 | 0 | 0 | 197 |

The candidate did execute other standard strings rows:
`489` rows total in candidate `all-results.txt`, with `407 PASSED`,
`39 FAILED`, `42 SKIPPED`, and `1 BORKED`. Those 489 rows are not the 197
assigned regressions. The assigned rows have no row-level candidate failure
diffs because they are absent from the candidate result/status files.

## Function clusters

Clusters are inferred from PHPT paths plus `--TEST--` titles in the pinned
php-src checkout:

| Cluster | Count | Examples |
| --- | ---: | --- |
| substring/search/compare `str*` core | 48 | `substr_compare`, `strcspn`, `stripos`, `strrpos`, `strtok`, `str_contains` |
| printf/sprintf/fprintf/v*printf/sscanf formatting | 45 | `sprintf_variation3`, `vfprintf_basic7_64bit`, `sscanf_basic7` |
| replace/translate/split/join/repeat/shuffle | 25 | `str_replace_basic`, `strtr_with_reference`, `chunk_split_basic`, `implode_error` |
| binary/hash/encoding/phonetic/edit-distance | 24 | `bin2hex`, `md5`, `crypt_blowfish_variation1`, `pack_Z`, `levenshtein_bug_6562` |
| HTML/entity/charset/UTF conversion | 11 | `htmlentities10`, `html_entity_decode_cp866`, `get_html_translation_table_basic1` |
| escaping/slashes/tags | 11 | `addcslashes_001`, `strip_tags`, `stripslashes_variation3` |
| path/file/source helpers | 7 | `basename_basic`, `basename_invalid_path`, `dirname_multi`, `highlight_file` |
| parse_str/string increment/cast/nl2br | 7 | `parse_str_null_bytes`, `str_increment_errors`, `strval_variation2` |
| trim/pad/case/word wrapping | 6 | `ltrim_basic`, `rtrim_basic`, `ucwords_basic`, `wordwrap_basic` |
| legacy numbered/basic coverage | 1 | `005.phpt` |
| unknown/bug title does not cleanly name one function | 12 | named below |

Named unknown or mixed semantic cases that should stay explicit until replay:
`bug22187` (`number_format()` crash), `bug26878` (mixed references/types),
`bug27675` and `bug33076` (`str_ireplace` shrinking/counting crashes),
`bug40754` (generic overflow checks inside string functions), `bug53021`
(numeric entity conversion), `bug62462` (quoted-printable multibyte soft
break), `bug72152` (`base64_decode` strict NUL detection), `gh11982`
(`str_getcsv` NUL behavior), `oss_fuzz_57392` (`fgetcsv` NUL delimiter),
`strrev`, and `strrev_variation1`.

## Shard and harness symptoms

- Candidate `counts.tsv`: `7197` passed, `8851` failed, `2222` skipped,
  `669` borked, `2` warned, public comparable score `7197/20294`.
- Candidate `pass-regression-summary.tsv`: baseline passes `7869`,
  current passes `7196`, pass regressions `1166`.
- Candidate `shard-exit-codes.tsv`: all six shards exited with rc `1`.
- Candidate `shard-signal-summary.tsv`: no signal exit was recorded
  (`signal_count=0`).
- Candidate `invalid-proof-marker-summary.tsv`: `invalid_marker_hits=0`.
- Candidate `aggregate-warnings.tsv`: `missing_results=0`, so aggregation
  saw result files, but those files did not contain the assigned rows.
- Candidate shard result coverage for standard strings:
  - shard 01: 122 strings rows (`101 PASS`, `10 FAIL`, `10 SKIP`, `1 BORK`)
  - shard 02: 123 strings rows (`93 PASS`, `15 FAIL`, `15 SKIP`)
  - shard 03: 0 strings rows
  - shard 04: 0 strings rows
  - shard 05: 122 strings rows (`106 PASS`, `7 FAIL`, `9 SKIP`)
  - shard 06: 122 strings rows (`107 PASS`, `7 FAIL`, `8 SKIP`)
- Shards 03 and 04 have `results.txt`, `stdout.log`, and empty `stderr.log`,
  but no copied `run-tests.log`. Their stdout contains a concrete redirect
  harness abort:
  - shard 03: `ERROR: cannot open directory:
    .../run-tests-harnesses/shard-03/ext/pdo/tests`
  - shard 04: `ERROR: cannot open directory:
    .../run-tests-harnesses/shard-04/ext/pdo/tests`

The strongest bucket for the 197 assigned regressions is therefore
`candidate result missing/not captured`, not demonstrated string-function
semantic failure. Shard 03/04 aborts explain zero strings coverage in those
shards. For missing rows that would otherwise be expected in shards with
strings coverage, the current artifacts do not include enough row-level
diagnostics to distinguish run-tests redirect/list mutation from result
capture omission. Keep those as `unknown harness/list interaction` until
focused replay.

## Representative rows

| Row | Accepted status | Candidate status | Diagnostic note |
| --- | --- | --- | --- |
| `php-src/ext/standard/tests/strings/bin2hex.phpt` | `PASSED` | missing from `all-results.txt` and `current-status.normalized.tsv` | Simple binary encoder row. Candidate did run sibling `bin2hex_001.phpt` as `PASSED`, so this row is not evidence that `bin2hex()` globally regressed. |
| `php-src/ext/standard/tests/strings/basename_invalid_path.phpt` | `PASSED` | missing from both candidate status files | Path helper row with invalid byte paths. Candidate did run other basename/path strings rows, but not this accepted pass row. |
| `php-src/ext/standard/tests/strings/md5.phpt` | `PASSED` | missing from both candidate status files | Hash/encoding row. Candidate shard logs contain unrelated `ext/hash/tests/md5.phpt` as `PASSED`; this row still has no candidate result. |
| `php-src/ext/standard/tests/strings/sprintf_variation3.phpt` | `PASSED` | missing from both candidate status files | Formatting row. Candidate did run other `sprintf_*` rows, including `sprintf_variation34_64bit.phpt` as `PASSED`, so this row needs focused replay before any semantic claim. |
| `php-src/ext/standard/tests/strings/strtr_with_reference.phpt` | `PASSED` | missing from both candidate status files | Reference-sensitive translate row. This remains an unknown semantic case because no candidate output exists for it. |

## Likely buckets

- Harness/control-plane: all 197 assigned rows are pass regressions only
  because they disappeared from candidate pass/status output.
- Confirmed shard harness abort: shard 03/04 `run-tests.php` redirect handling
  cannot find `ext/pdo/tests` under the copied shard harness directory.
- Unknown harness/list interaction: some assigned rows are absent despite other
  strings rows appearing in candidate shards; current artifacts do not archive
  the shard test-list files, and the `/tmp` run root was gone during this
  audit.
- Semantic status unknown: none of the 197 rows has a candidate FAIL/SKIP/BORK
  diagnostic. Do not claim compiler/runtime support or regression cause from
  these rows until focused replay produces executable evidence.
- Out of scope for this report: the 39 candidate failed strings rows that are
  present in candidate status files, because they are not in the 197 accepted
  PASS regression rows assigned to lane 15.

## Focused replay recommendations

Do not run a full PHPT gate for this lane. Use only the existing gate script
and environment conventions recorded in the candidate artifacts:

1. Build a focused list from the artifact, not by hand:
   `regressions-from-latest-published-passes.txt` filtered to
   `php-src/ext/standard/tests/strings/`, translated to the pinned
   `/home/claude/php-src-phpt/...` paths.
2. Reuse the candidate `run_gate.sh` environment values:
   `WRAPPER=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper`,
   `PHPT_SYSTEM_PHP=php`, `PHPC_PHPT_TIMEOUT_SECONDS=55`,
   `PHPC_PHPT_KILL_AFTER_SECONDS=5`, `NO_INTERACTION=1`,
   `CARGO_BUILD_JOBS=1`, and `CARGO_INCREMENTAL=0`.
3. Do not point replay at the stale candidate `PHPC_BIN` path under
   `/tmp/phpt-full-current-score-20260604T221205Z-...`; that run root was no
   longer present during this audit. Rebuild or reuse a manager-provided
   binary for exact `56fe9377fb46be00db5fdd30c966fdba406dc581` before replay.
4. First replay the five representative rows above as a smoke check. If they
   all produce statuses, replay the full 197-row focused list and record
   `all-results.txt`, normalized statuses, and run-tests log as a new focused
   artifact.
5. Separately reproduce the shard 03/04 redirect abort with the existing
   `run_gate.sh` harness layout before changing compiler/runtime behavior.
   The failure path is in the copied shard harness, not in
   `php-src/ext/standard/tests/strings` itself.
