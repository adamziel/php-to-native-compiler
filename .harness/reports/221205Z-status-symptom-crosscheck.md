# 221205Z Status/Symptom Cross-Check

This is diagnostic/control-plane work only. It does not implement compiler or
runtime behavior, does not run a full PHPT gate, and cannot move the public PHPT
score by itself.

Artifacts inspected:

- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt`
- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv`
- `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/all-results.txt`
- shard stdout/results logs below the same evidence directory

## Summary

`regressions-from-latest-published-passes.txt` contains exactly 1,166 unique
regression rows.

Only 30 of those 1,166 rows have direct
`current-status.normalized.tsv` entries:

| Current status | Rows |
| --- | ---: |
| `FAILED` | 27 |
| `BORKED` | 3 |

The same 30 rows are present in `all-results.txt` with the same status counts.
There are no regression rows that are present only in `all-results.txt` or only
in shard stdout while absent from `current-status.normalized.tsv`.

The remaining 1,136 regression rows do not have direct aggregate status rows:

| Classification | Rows |
| --- | ---: |
| Direct `current-status.normalized.tsv` entry | 30 |
| Represented only in `all-results.txt`/shard logs, absent from current-status | 0 |
| Absent from current-status/all-results/shard per-test evidence | 630 |
| Ambiguous because the assigned shard aborted with no per-test row | 506 |
| Total regressions | 1,166 |

The ambiguous rows are on the two aborted shards:

| Shard | Abort symptom | Missing regression rows assigned to shard |
| --- | --- | ---: |
| `03` | `ERROR: cannot open directory: .../run-tests-harnesses/shard-03/ext/pdo/tests` | 199 |
| `04` | `ERROR: cannot open directory: .../run-tests-harnesses/shard-04/ext/pdo/tests` | 307 |

The result files existed and were aggregated (`aggregate-warnings.tsv` says
`missing_results 0`), but shard stdout shows these two shards did not reach
`Report saved to:`. For regression rows assigned to those shards that have no
current-status/all-results/stdout row, the current artifacts cannot distinguish
compiler behavior from missing evidence after the shard-level control-plane
abort.

The 630 absent rows are baseline-pass rows that are neither current passes nor
direct aggregate status rows in the available 221205Z evidence. Their planned
shards either completed with `Report saved to:` or otherwise produced no
per-test status for those specific paths. These rows cannot be classified as
direct `FAILED`/`BORKED` from the current artifacts.

## Why 30 Is Much Smaller Than 1,166

The pass-regression blocker is computed by set subtraction:

```text
comm -23 baseline-passes.normalized.txt current-passes.normalized.txt
```

So a test that passed in the latest accepted baseline counts as a regression
whenever it is not present in `current-passes.normalized.txt`. It does not need
to have a direct `FAILED` or `BORKED` row.

`current-status.normalized.tsv` is generated from `all-results.txt`:

```text
awk -F '\t' 'NF >= 2 {p=$2; sub(/^# /,"",p); sub(/^.*\/php-src\//,"php-src/",p); print $1 "\t" p}' all-results.txt | sort -u
```

That means direct current status only exists for tests that reached
`all-results.txt`. In this 221205Z run, only 30 of the 1,166 baseline-pass
regressions reached the aggregate as direct `FAILED`/`BORKED` rows. The other
1,136 are still gate-blocking pass regressions because they are absent from
current passes, but most do not have direct aggregate failure-status evidence.

## Representative Rows

| Regression row | Mapping evidence |
| --- | --- |
| `php-src/ext/bcmath/tests/number/properties_unset.phpt` | Direct aggregate failure. Planned shard `03`, position `915/3630`. `current-status.normalized.tsv:671` is `FAILED`; `all-results.txt:8312` is `FAILED`. |
| `php-src/ext/intl/tests/rangeformatter/rangeformatter_icu63_compatibility.phpt` | Direct aggregate bork. Planned shard `06`, position `1482/3630`. `current-status.normalized.tsv:258` is `BORKED`; `all-results.txt:16891` is `BORKED`. |
| `php-src/ext/posix/tests/001.phpt` | No current-status/all-results/stdout row. Assigned to shard `03`, which aborted with `ERROR: cannot open directory: .../shard-03/ext/pdo/tests` and did not reach `Report saved to:`. Ambiguous due aborted shard evidence. |
| `php-src/ext/posix/tests/bug75696.phpt` | No current-status/all-results/stdout row. Assigned to shard `04`, which aborted with `ERROR: cannot open directory: .../shard-04/ext/pdo/tests` and did not reach `Report saved to:`. Ambiguous due aborted shard evidence. |
| `php-src/ext/phar/tests/bug79797.phpt` | No current-status/all-results/stdout row. Planned shard `06`, position `2059/3630`; shard `06` reached `Report saved to:`. This is an absent aggregate row, not a direct `FAILED`/`BORKED` row in the available evidence. |

## Deterministic Mapping Method

Low-CPU inspection only was used. No PHPT gate was run.

Commands and pseudocode:

```sh
wc -l \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/regressions-from-latest-published-passes.txt \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/all-results.txt

cut -f1 \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/current-status.normalized.tsv |
  sort | uniq -c | sort -nr

rg -n 'ERROR: cannot open directory|Report saved to:' \
  /home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377/shard-*/stdout.log
```

```python
def norm(path):
    # Convert absolute /tmp/.../php-src/... paths to php-src/... keys.
    return path[path.index("php-src/"):]

regressions = set(lines("regressions-from-latest-published-passes.txt"))
current = {norm(path): status for status, path in tsv("current-status.normalized.tsv")}
all_results = {norm(path): status for status, path in tsv("all-results.txt")}

direct = [r for r in regressions if r in current]
direct_counts = Counter(current[r] for r in direct)

# Reconstruct run_gate.sh shard assignment:
# Use the clean f97ff59 Git tree and shell-sort ordering, matching run_gate.sh's
# clean clone plus `find ... | sort`.
all_tests = shell_sort(git_ls_tree("f97ff59", suffix=".phpt"))
serialized = shell_sort([p for p in all_tests if p.startswith("tests/security/open_basedir_")])
sharded = [p for p in all_tests if p not in serialized]
for nr, path in enumerate(sharded, start=1):
    shard = ((nr - 1) % 6) + 1
    position = ((nr - 1) // 6) + 1
    planned[norm(path)] = (shard, position)

# Parse shard stdout for abort symptoms and report completion.
aborted_shards = {"03", "04"}  # both have ERROR: cannot open directory .../ext/pdo/tests
completed_shards = {"01", "02", "05", "06"}  # these reached "Report saved to:"

for r in regressions:
    if r in current:
        classify("direct current-status row")
    elif r in all_results or r in shard_stdout_seen:
        classify("represented only outside current-status")
    elif planned[r].shard in aborted_shards:
        classify("ambiguous due aborted shard/missing per-test evidence")
    else:
        classify("absent from aggregate/shard evidence")
```

No compiler/runtime/source files were edited. No full PHPT gate was run.
