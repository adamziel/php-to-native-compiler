# 221205Z Standard Claim vs Regression Audit

Lane: 35, developer-83

Scope: read-only M0 audit for standard-library support claims against the
blocked `221205Z` public PHPT gate. This report identifies which public
README/support-matrix claims are put at risk by the standard regression bucket,
and separates claim risk from proven semantic contradiction. No compiler,
runtime, or public-support documentation was edited.

## Evidence

- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate status artifact:
  `current-status.normalized.tsv`
- PHPT source checkout:
  `/home/claude/php-src-phpt`
- Public support documents:
  `README.md`, `docs/SUPPORT.md`

The candidate gate is blocked at `7197 / 20294 = 35.46%` public score with
`1166` latest-public PASS regressions. The accepted baseline referenced by the
gate is
`/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt`.

## Public Claim Surface

I did not find a broad claim that the project supports the full PHP standard
library. The relevant public claim is narrower:

- `README.md:629` says the runtime has a documented builtin subset for strings,
  arrays, constants, filesystem/request-state probes, output-buffer probes, type
  checks, callability checks, bounded truthy assertions, object/class metadata,
  and debug-style output.
- `docs/SUPPORT.md:3104-3125` enumerates many standard builtins in that subset,
  including array helpers, string helpers, local filesystem/stream functions,
  directory-handle functions, `assert`, type checks, callability checks,
  constants, and debug output.
- Detailed `docs/SUPPORT.md` entries keep these claims bounded. Examples:
  `array_unshift()` documents direct-variable and selected callback forms,
  `str_replace()` excludes replacement arrays, subject arrays, nested searches,
  indirect count output, exact warnings, binary edge cases, and native lowering.

This means the 221205Z standard regression bucket should not be treated as a
contradiction of a full-stdlib claim. It is a contradiction risk against the
bounded documented subset if a row replays as a candidate semantic failure
inside the documented slice.

## Standard Regression Shape

Joining `regressions-from-latest-published-passes.txt` to
`current-status.normalized.tsv` gives this standard-extension status:

| Status | Rows |
| --- | ---: |
| `ABSENT` from candidate status artifact | 792 |
| `FAILED` in candidate status artifact | 2 |
| Total standard regressions | 794 |

The only preserved `FAILED` rows are:

- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt`
- `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt`

All other standard regressions are artifact-absent in the candidate gate. The
right first conclusion is therefore claim-risk pending replay, not direct
semantic failure of every standard builtin.

Subdirectory distribution:

| Standard area | Rows | Candidate status |
| --- | ---: | --- |
| `array` | 249 | 249 `ABSENT` |
| `strings` | 197 | 197 `ABSENT` |
| `file` | 160 | 160 `ABSENT` |
| `math` | 53 | 53 `ABSENT` |
| `general_functions` | 44 | 44 `ABSENT` |
| `dir` | 14 | 14 `ABSENT` |
| `serialize` | 14 | 14 `ABSENT` |
| `class_object` | 12 | 12 `ABSENT` |
| `streams` | 10 | 10 `ABSENT` |
| `url` | 10 | 10 `ABSENT` |
| `http` | 7 | 7 `ABSENT` |
| `assert` | 6 | 6 `ABSENT` |
| `directory` | 6 | 4 `ABSENT`, 2 `FAILED` |
| `network` | 3 | 3 `ABSENT` |
| single-row top-level standard files | 9 | 9 `ABSENT` |

## Claim-Risk Buckets

| Area | Public claim overlap | Regression evidence | Claim status |
| --- | --- | --- | --- |
| Arrays | `docs/SUPPORT.md` enumerates `array_chunk`, `array_merge`, `array_filter`, `array_map`, `array_count_values`, `array_diff`, `array_unique`, `array_unshift`, `next`, `ksort`, `in_array`, `array_search`, and more. | 249 array rows, all artifact-absent. Prior selector chose eight low-dependency replay rows including `array_chunk2`, `array_count_values`, `array_filter_basic`, `array_map_basic`, `array_merge`, and `array_multisort_basic1`. | High claim-risk if focused replay produces semantic failures inside documented slices. Not a direct contradiction while rows are absent from the candidate artifact. |
| Strings | `README.md` and `docs/SUPPORT.md` claim a bounded string builtin subset. `docs/SUPPORT.md:3487` documents `str_replace()` specifically. | 197 string rows, all artifact-absent. Lexical scan finds documented functions such as `str_replace`, `substr`, `strncmp`, `stripos`, and `strrpos` in regressed PHPTs; representative rows include `strings/str_replace_basic.phpt`, `strings/str_replace_array_refs.phpt`, and `strings/bug71188.phpt`. | High claim-risk for exact supported string builtins. Replay must separate supported scalar cases from documented unsupported arrays, objects/resources, warning, and binary-edge cases. |
| Filesystem, streams, and directories | README lists interpreter-only local file/stream/directory support; `docs/SUPPORT.md:3108-3111` enumerates `file_get_contents`, `file_put_contents`, `fopen`, `fwrite`, `fscanf`, `fread`, `rewind`, `stream_get_contents`, `feof`, `ftell`, `fseek`, `ftruncate`, `fstat`, `fclose`, `opendir`, `readdir`, `closedir`, `filesize`, `filemtime`, and related metadata helpers. | 160 `file`, 10 `streams`, 14 `dir`, and 6 `directory` rows. Of these, 188 are artifact-absent and two `directory` readonly rows are preserved failures. Lexical scan shows common helpers including `fopen`, `fclose`, `fwrite`, `rewind`, `feof`, `ftell`, `file_get_contents`, `filesize`, `fscanf`, and `stream_context_create`. | High claim-risk for local-file and stream slices if replay fails semantically. The two readonly `Directory` failures are immediate narrow failures, but they do not directly contradict the documented local stream/directory helper subset unless docs claim exact readonly internal-property parity. |
| Debug output and scalar/type helpers | README claims debug-style output, type checks, callability checks, constants, and assertions; `docs/SUPPORT.md` enumerates `var_dump`, `print_r`, `gettype`, `is_*`, `is_callable`, `define`, `constant`, `defined`, and `assert`. | `general_functions` has 44 artifact-absent rows, `assert` has 6 artifact-absent rows, `class_object` has 12 artifact-absent rows, and many standard PHPTs mention `var_dump` as a display helper. | Medium to high claim-risk. Lexical `var_dump` counts are not proof that debug output is the semantic target because PHPTs often use it for expected-output formatting. Replay should prioritize rows whose titles/main sections target the claimed builtin. |
| HTTP/header/request state | README documents bounded `header`, `headers_list`, `headers_sent`, `http_response_code`, `setcookie`, and `setrawcookie` paths; `docs/SUPPORT.md` includes the same family in the builtin subset. | `http` has 7 artifact-absent rows, plus top-level `setcookie_samesite_validation.phpt` and `setrawcookie_basic_001.phpt` are artifact-absent regressions. | Medium claim-risk. These rows are directly relevant to public header/cookie claims if replay reaches supported CLI header-state paths rather than SAPI-specific or unsupported option behavior. |
| Math, URL, serialization, and misc standard rows | `docs/SUPPORT.md` claims selected scalar helpers such as `min`, `rand`, `md5`, `version_compare`, `parse_str`, and URL/string parsing helpers, but not broad ext/standard parity. | 53 `math`, 14 `serialize`, 10 `url`, and several single-row files are all artifact-absent. | Mixed claim-risk. Row-level replay is required before treating these as support-doc contradictions because many may target unclaimed PHP behaviors or edge-case diagnostics. |

## Lexical Scan Notes

A lexical scan of the 794 standard regressions against the documented builtin
names found many overlaps, including:

| Builtin mention | Rows |
| --- | ---: |
| `var_dump` | 674 |
| `fopen` | 140 |
| `fclose` | 126 |
| `unlink` | 113 |
| `substr` | 66 |
| `mkdir` / `rmdir` | 57 each |
| `count` | 49 |
| `clearstatcache` | 38 |
| `fwrite` | 37 |
| `rewind` | 35 |
| `feof` | 30 |
| `ftell` | 28 |
| `file_get_contents` | 24 |
| `filesize` | 23 |
| `print_r` | 18 |
| `fscanf` | 13 |
| `array_chunk` | 11 |
| `in_array` | 10 |
| `array_map` | 7 |
| `str_replace` | 6 |
| `assert` | 6 |
| `array_diff` / `array_unshift` | 5 each |

This scan is only a prioritization aid. PHPTs frequently use setup/cleanup
helpers and `var_dump()`/`print_r()` to display values. A lexical mention should
not be recorded as a support contradiction without replay and source-section
inspection.

## Contradiction Classification

Current public docs are not contradicted by a broad-stdlib claim because no such
claim was found. The blocked candidate must still not be promoted as a public
score improvement while these rows regress.

For standard-library claims, the defensible classification is:

- `792` rows: support-claim risk pending replay because they are absent from the
  candidate status artifact.
- `2` rows: narrow preserved failures in `DirectoryClass_readonly_*`; these are
  immediate failures for readonly internal property behavior, not a broad local
  directory-helper contradiction.
- `0` rows: proven full-stdlib contradiction, because the docs only claim a
  bounded subset and most rows lack preserved candidate semantic output.

If focused replay turns representative rows into semantic failures inside the
documented support slice, the next owner should either fix implementation and
tests or narrow `README.md`, `docs/SUPPORT.md`, and `docs/PROGRESS.md` with the
exact unsupported edge cases. Until then, do not downgrade public docs based on
artifact-absent rows alone.

## Recommended Follow-Up

1. Replay the existing standard array selector first:
   `array_chunk2`, `array_count_values`, `array_filter_basic`,
   `array_map_basic`, `array_merge`, `array_walk_basic1`, and
   `array_multisort_basic1`.
2. Add a string-focused replay slice around exact documented string claims:
   `strings/str_replace_basic.phpt`, `strings/str_replace_array_refs.phpt`,
   `strings/strncmp_basic.phpt`, `strings/stripos.phpt`, and
   `strings/strrpos.phpt`.
3. Add a local file/stream replay slice that avoids broad environment/SAPI
   dependencies first: `file/fseek_ftell_rewind_basic1.phpt`,
   `file/fscanf_variation11.phpt`, `streams/stream_get_meta_data_file_variation1.phpt`,
   and a small `file_get_contents`/`file_put_contents` pair.
4. Keep `DirectoryClass_readonly_handle.phpt` and
   `DirectoryClass_readonly_path.phpt` as a separate diagnostics/internal-class
   repair lane.

## Commands Run

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter, defaultdict

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
reg = root / 'regressions-from-latest-published-passes.txt'
status_file = root / 'current-status.normalized.tsv'

statuses = {}
for line in status_file.read_text().splitlines():
    parts = line.split('\t')
    if len(parts) >= 2 and parts[1].startswith('php-src/'):
        statuses[parts[1]] = parts[0]

counts = Counter()
subdirs = defaultdict(Counter)
for test in reg.read_text().splitlines():
    if not test.startswith('php-src/ext/standard/tests/'):
        continue
    status = statuses.get(test, 'ABSENT')
    rel = test.removeprefix('php-src/ext/standard/tests/')
    subdir = rel.split('/', 1)[0]
    counts[status] += 1
    subdirs[subdir][status] += 1

print(counts)
for subdir, sub_counts in sorted(subdirs.items(), key=lambda item: (-sum(item[1].values()), item[0])):
    print(subdir, sum(sub_counts.values()), dict(sub_counts))
PY
```

```sh
rg -n "builtin subset|array_chunk|str_replace|fopen|setcookie|assert|var_dump|print_r" \
  README.md docs/SUPPORT.md docs/PROGRESS.md docs/ARCHITECTURE.md
```

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter, defaultdict
import re

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
php_src = Path('/home/claude/php-src-phpt')
rows = [
    row for row in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines()
    if row.startswith('php-src/ext/standard/tests/')
]
funcs = ['var_dump', 'fopen', 'fclose', 'unlink', 'substr', 'mkdir', 'rmdir',
         'count', 'clearstatcache', 'fwrite', 'rewind', 'feof', 'ftell',
         'file_get_contents', 'filesize', 'print_r', 'fscanf', 'array_chunk',
         'in_array', 'array_map', 'str_replace', 'assert', 'array_diff',
         'array_unshift']
patterns = {name: re.compile(r'(?<![A-Za-z0-9_])' + re.escape(name) + r'\s*\(')
            for name in funcs}
counts = Counter()
examples = defaultdict(list)
for row in rows:
    path = php_src / row.removeprefix('php-src/')
    text = path.read_text(errors='ignore')
    for name, pattern in patterns.items():
        if pattern.search(text):
            counts[name] += 1
            examples[name].append(row)

print(counts.most_common())
PY
```
