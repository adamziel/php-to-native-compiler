# First Repair Lane Proposals

Lane: 36, developer-83

Scope: read-only M0 proposal draft for the first narrow repair lanes that
should wait for shard/replay evidence from the blocked `221205Z` PHPT gate. No
compiler/runtime source edits were made.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate status artifact:
  `current-status.normalized.tsv`
- Existing local reports:
  `.harness/reports/regression-repair-backlog-template.md`,
  `.harness/reports/standard-array-replay-selector.md`,
  `.harness/reports/221205Z-standard-strings-replace-replay.md`,
  `.harness/reports/221205Z-standard-scalar-misc.md`,
  `.harness/reports/221205Z-secondary-ext.md`,
  `.harness/reports/221205Z-source-diff-risk.md`,
  `.harness/reports/standard-claim-vs-regression.md`

The candidate remains blocked at `7197 / 20294 = 35.46%` public score with
`1166` latest-public PASS regressions. Current regression status shape is:

| Candidate status | Rows |
| --- | ---: |
| `ABSENT` | 1136 |
| `FAILED` | 27 |
| `BORKED` | 3 |
| Total | 1166 |

The proposals below intentionally target absent-result clusters. They are
candidate repair lanes only after focused accepted-vs-candidate replay or shard
evidence proves a semantic failure. If the selected precheck rows remain absent
from candidate result/status artifacts, route the work to M0/M1
result-normalization/control-plane instead of implementation.

## Top Absent Clusters

| Cluster | Regressions | Candidate status |
| --- | ---: | --- |
| `php-src/ext/standard/tests/array` | 249 | 249 `ABSENT` |
| `php-src/ext/standard/tests/strings` | 197 | 197 `ABSENT` |
| `php-src/ext/standard/tests/file` | 160 | 160 `ABSENT` |
| `php-src/ext/spl/tests` | 137 | 137 `ABSENT` |
| `php-src/ext/reflection/tests` | 110 | 110 `ABSENT` |
| `php-src/ext/uri/tests` | 41 | 41 `ABSENT` |
| `php-src/ext/posix/tests` | 16 | 16 `ABSENT` |
| `php-src/ext/standard/tests/dir` | 14 | 14 `ABSENT` |
| `php-src/ext/standard/tests/serialize` | 14 | 14 `ABSENT` |
| `php-src/ext/tokenizer/tests` | 14 | 14 `ABSENT` |
| `php-src/ext/standard/tests/streams` | 10 | 10 `ABSENT` |
| `php-src/ext/standard/tests/url` | 10 | 10 `ABSENT` |

Direct `FAILED` and `BORKED` rows, such as `DirectoryClass_readonly_*`,
`DatePeriod_*`, `xmlreader/014.phpt`, and `SKIPIF` constant rows, should remain
separate M0-direct lanes. They are useful, but they do not explain the five
large absent clusters below.

## Proposal 1: Standard Array Builtin Replay-To-Repair

| Field | Value |
| --- | --- |
| Readiness | `M0-replay` first; promote to `M2-repair` only after replay gives preserved semantic failures. |
| Owner module | Interpreter standard array builtins and runtime array value semantics. |
| Evidence | `249` standard array regressions, all absent from candidate status artifacts. |
| Precheck PHPTs | `php-src/ext/standard/tests/array/array_chunk2.phpt`; `php-src/ext/standard/tests/array/array_count_values.phpt`; `php-src/ext/standard/tests/array/array_diff_single_array.phpt`; `php-src/ext/standard/tests/array/array_filter_basic.phpt`; `php-src/ext/standard/tests/array/array_map_basic.phpt`; `php-src/ext/standard/tests/array/array_merge.phpt`; `php-src/ext/standard/tests/array/array_walk/array_walk_basic1.phpt`; `php-src/ext/standard/tests/array/sort/array_multisort_basic1.phpt`. |
| Low-risk reason | Each row is a focused no-`SKIPIF` selector from the standard-array replay report and maps to a concrete builtin family rather than a broad standard-library rewrite. |
| High-yield reason | Array rows are the largest single absent cluster (`249` rows), and successful semantic repair could remove several callback, key-ordering, mutation, and sort-flag subclusters. |
| Wait condition | Do not open implementation until replay shows candidate semantic failures with preserved output. If the rows stay absent, fix result collection/classification first. |

## Proposal 2: Standard String Replacement/Search Builtins

| Field | Value |
| --- | --- |
| Readiness | `M0-replay` first; promote only for semantic failures inside documented string builtin slices. |
| Owner module | Interpreter string builtins and scalar string coercion diagnostics. |
| Evidence | `197` standard string regressions, all absent. The replacement selector found strict `str_replace()` / `str_ireplace()` PASS-regression rows. |
| Precheck PHPTs | `php-src/ext/standard/tests/strings/str_replace_basic.phpt`; `php-src/ext/standard/tests/strings/str_replace_array_refs.phpt`; `php-src/ext/standard/tests/strings/bug27675.phpt`; `php-src/ext/standard/tests/strings/strncmp_basic.phpt`; `php-src/ext/standard/tests/strings/stripos_variation3.phpt`; `php-src/ext/standard/tests/strings/strrpos_offset.phpt`. |
| Low-risk reason | The first three rows are already selected for replacement replay; the adjacent search/compare rows are small direct string builtin targets. Keep one function family per implementation lane after replay identifies the failing mechanism. |
| High-yield reason | Strings are the second-largest standard cluster (`197` rows). The docs claim a bounded string subset, so confirmed semantic failures here are high value for score and support accuracy. |
| Wait condition | If replay classifies `str_replace_basic.phpt` and `bug27675.phpt` as absent-result artifacts, do not start string runtime work. If only reference-backed or unsupported array cases fail, narrow docs/unsupported edges rather than broadening the fix. |

## Proposal 3: Local File, Stream, and Directory Helper Tranche

| Field | Value |
| --- | --- |
| Readiness | `M0-replay` first; split by filesystem, stream resource, or directory handle after semantic evidence. |
| Owner module | Interpreter filesystem builtins, stream-resource runtime, and directory-handle runtime. |
| Evidence | `160` `file` rows, `10` `streams` rows, `14` `dir` rows, and `4` artifact-absent `directory` rows. |
| Precheck PHPTs | `php-src/ext/standard/tests/file/fseek_ftell_rewind_basic1.phpt`; `php-src/ext/standard/tests/file/fscanf_variation11.phpt`; `php-src/ext/standard/tests/file/file_get_contents_file_put_contents_variation1.phpt`; `php-src/ext/standard/tests/streams/stream_get_meta_data_file_variation1.phpt`; `php-src/ext/standard/tests/dir/opendir_basic.phpt`; `php-src/ext/standard/tests/directory/directory_constants.phpt`. |
| Low-risk reason | These rows exercise local temp files, stream metadata, and directory iteration without requiring a full SAPI, network, or extension environment. A replay-confirmed fix can stay within one helper family at a time. |
| High-yield reason | The combined local file/stream/directory absent surface is about `188` rows, close to the string cluster size, and overlaps public claims for local filesystem and stream helpers. |
| Wait condition | Replay must distinguish semantic helper failures from PHPT environment/setup or missing-result normalization. Keep readonly `DirectoryClass_readonly_*` diagnostics in a separate direct-failure lane. |

## Proposal 4: SPL Object And Iterator Runtime Tranche

| Field | Value |
| --- | --- |
| Readiness | `M0-replay` first; promote only after a specific SPL class/method mechanism fails semantically. |
| Owner module | SPL runtime classes, iterator behavior, object storage, and autoload/class lookup. |
| Evidence | `137` SPL regressions, all absent. Largest subclusters are `ArrayObject` (`29`), `SplFileObject` (`19`), `SplObjectStorage` (`9`), and autoloading (`7`). |
| Precheck PHPTs | `php-src/ext/spl/tests/SplFileObject/SplFileObject_fgetcsv_basic.phpt`; `php-src/ext/spl/tests/SplFileObject/SplFileObject_key_basic.phpt`; `php-src/ext/spl/tests/ArrayObject/arrayObject___construct_basic1.phpt`; `php-src/ext/spl/tests/ArrayObject/arrayObject_exchangeArray_basic1.phpt`; `php-src/ext/spl/tests/SplObjectStorage/SplObjectStorage_current_empty_storage.phpt`; `php-src/ext/spl/tests/autoloading/spl_autoload_call_basic.phpt`. |
| Low-risk reason | The prechecks isolate one method/class family each and avoid using `eval`-dependent autoload rows as first implementation evidence. |
| High-yield reason | SPL is the largest non-standard absent cluster (`137` rows), and source-diff risk points at SPL file-object and autoload/class-order changes. |
| Wait condition | If replay confirms only `SplFileObject` failures, open a `SplFileObject` lane and leave `ArrayObject`/autoload untouched. If autoload rows involving `eval` are implicated, keep them late-priority unless a manager opens that work. |

## Proposal 5: Reflection Metadata Descriptor Tranche

| Field | Value |
| --- | --- |
| Readiness | `M0-replay` first; promote after replay identifies one descriptor family. |
| Owner module | Reflection class/function/method/property/parameter metadata. |
| Evidence | `110` reflection regressions, all absent. Source-diff risk includes reflection parameter metadata, enum reflection, generated methods, typed constants, and property metadata changes. |
| Precheck PHPTs | `php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt`; `php-src/ext/reflection/tests/ReflectionClass_getProperties_001.phpt`; `php-src/ext/reflection/tests/ReflectionParameter_001.phpt`; `php-src/ext/reflection/tests/internal_parameter_default_value/ReflectionParameter_getDefaultValue_Internal.phpt`; `php-src/ext/reflection/tests/ReflectionMethod_constructor_basic.phpt`; `php-src/ext/reflection/tests/ReflectionProperty_getModifiers.001.phpt`. |
| Low-risk reason | Each row maps to a concrete metadata descriptor. A repair can be constrained to class constants, properties, parameters, methods, or modifiers instead of broad reflection behavior. |
| High-yield reason | Reflection is the second-largest non-standard absent cluster (`110` rows) and has direct source-diff overlap, so confirmed semantic failures would be actionable. |
| Wait condition | Do not mix `bug64936.phpt` or other `eval` rows into the first implementation lane. If prechecks replay as absent, keep work in the shard/control-plane lane. |

## Proposal Ordering

1. Run shard evidence/replay for one representative row from each proposal
   before opening any M2 repair lane.
2. If all five samples remain absent from result/status artifacts, the first
   real repair lane is M1 control-plane/result normalization, not product code.
3. If only one sample becomes a preserved semantic failure, open the
   corresponding narrow lane and keep the other four proposals queued.
4. Do not claim public score movement from focused rows. A full pinned PHPT
   gate with zero unadjudicated latest-public PASS regressions is still required.

## Commands Run

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter, defaultdict

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows = [line.strip() for line in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]
status = {}
for line in (root / 'current-status.normalized.tsv').read_text().splitlines():
    parts = line.split('\t')
    if len(parts) >= 2 and parts[1].startswith('php-src/'):
        status[parts[1]] = parts[0]

print('overall', len(rows), Counter(status.get(row, 'ABSENT') for row in rows))
by_ext = defaultdict(list)
for row in rows:
    parts = row.split('/')
    if len(parts) > 3 and parts[0] == 'php-src' and parts[1] == 'ext':
        by_ext['ext/' + parts[2]].append(row)
for key, values in sorted(by_ext.items(), key=lambda item: (-len(item[1]), item[0]))[:12]:
    print(key, len(values), Counter(status.get(row, 'ABSENT') for row in values))
PY
```

```sh
python3 - <<'PY'
from pathlib import Path

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
regs = set((root / 'regressions-from-latest-published-passes.txt').read_text().splitlines())
prechecks = [
    'php-src/ext/standard/tests/array/array_chunk2.phpt',
    'php-src/ext/standard/tests/strings/str_replace_basic.phpt',
    'php-src/ext/standard/tests/file/fseek_ftell_rewind_basic1.phpt',
    'php-src/ext/standard/tests/streams/stream_get_meta_data_file_variation1.phpt',
    'php-src/ext/spl/tests/SplFileObject/SplFileObject_fgetcsv_basic.phpt',
    'php-src/ext/reflection/tests/ReflectionClass_getConstants_basic.phpt',
]
for row in prechecks:
    print(row, row in regs)
PY
```

```sh
sed -n '1,260p' .harness/reports/regression-repair-backlog-template.md
sed -n '1,180p' .harness/reports/standard-array-replay-selector.md
sed -n '1,180p' .harness/reports/221205Z-standard-strings-replace-replay.md
sed -n '1,220p' .harness/reports/221205Z-source-diff-risk.md
```
