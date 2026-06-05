# 221205Z Source-Diff Regression Risk Audit

Lane: 23, developer-83

Scope: read-only M0 audit comparing accepted `0b917f67` to blocked candidate
`56fe9377` and summarizing broad source changes that could explain the
standard/SPL/reflection regression shape. No compiler/runtime source edits were
made.

## Evidence

- Accepted public commit: `0b917f67a37d9ca9779d77f87173b628431c2425`
- Candidate public/source commit: `56fe9377fb46be00db5fdd30c966fdba406dc581`
- Candidate gate:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377`
- Baseline passes:
  `/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T135138Z-php-src-f97ff59-public-0b917f67-source-0b917f67/current-passes.normalized.txt`
- Regression list:
  `regressions-from-latest-published-passes.txt`
- Candidate status artifacts:
  `current-status.normalized.tsv`, `all-results.txt`, `pass-regression-summary.tsv`

The blocked candidate has raw public score `7197 / 20294`. Normalized latest
public PASS accounting is `7869` accepted baseline passes, `7196` candidate
passes, and `1166` pass regressions.

## Diff Size

The commit range is substantial:

| Measure | Value |
| --- | ---: |
| Commits in `0b917f67..56fe9377` | 40 |
| Files changed | 33 |
| Insertions | 15011 |
| Deletions | 1401 |

Largest changed files:

| File | Added | Removed | Risk note |
| --- | ---: | ---: | --- |
| `compiler/src/interpreter.rs` | 9412 | 651 | Main runtime/interpreter dispatch surface; direct risk for standard/SPL/reflection behavior. |
| `compiler/src/parser.rs` | 743 | 167 | New syntax and class-member parsing diagnostics. |
| `runtime/src/lib.rs` | 728 | 36 | Object/property visibility and GMP comparison runtime ABI changes. |
| `compiler/tests/object_model.rs` | 872 | 30 | Broad object-model behavior changes under test, but not production code. |
| `docs/SUPPORT.md` | 437 | 163 | Claim/support text churn; not an execution cause, but audit-relevant. |
| `Cargo.lock` | 560 | 0 | New dependency closure for crypt/password work. |

New direct dependencies in `compiler/Cargo.toml` are `bcrypt = "0.19.1"` and
`crypt3_rs = "0.1.1"`.

## Regression Weight

Largest regression buckets in the blocked gate:

| Bucket | Regressions | Candidate artifact status |
| --- | ---: | --- |
| `php-src/ext/standard/tests` | 794 | 792 `ABSENT`, 2 `FAILED` |
| `php-src/ext/spl/tests` | 137 | 137 `ABSENT` |
| `php-src/ext/reflection/tests` | 110 | 110 `ABSENT` |
| `php-src/ext/uri/tests` | 41 | 41 `ABSENT` |
| `php-src/ext/posix/tests` | 16 | 16 `ABSENT` |
| `php-src/ext/xmlreader/tests` | 9 | 8 `ABSENT`, 1 `FAILED` |

The most important source-diff conclusion is negative: the dominant
standard/SPL/reflection regressions are absent from candidate result/status
artifacts, not preserved as direct per-row failures. Source changes can explain
rows that replay as semantic failures, but source diff alone does not explain
why `792` standard rows, all `137` SPL rows, and all `110` reflection rows are
missing from candidate artifacts.

## Risk Areas

| Risk area | Source-diff evidence | Regression relevance | Priority |
| --- | --- | --- | --- |
| Candidate result/control-plane absence | Most pass regressions are absent from `current-status.normalized.tsv` and normalized `all-results.txt`. | Dominates standard/SPL/reflection accounting. This must be replayed before assigning broad implementation work. | Highest M0 |
| Object property visibility/readonly diagnostics | Parser/runtime changes for property hooks, asymmetric setter visibility, typed constants, readonly/property metadata, and scoped writes in `runtime/src/lib.rs`. | Direct preserved failures include `DirectoryClass_readonly_handle.phpt`, `DirectoryClass_readonly_path.phpt`, `DatePeriod_*readonly*`, `BcMath\\Number` property rows, and Zend readonly/property-hook rows. | High |
| SPL file object and class/autoload order | `a1d98e14` adds SPL memory file-object behavior; `087e3ad3` adds class-order autoload variance; interpreter adds `SplTempFileObject`, memory-stream helpers, and pending signature dependency autoload. | SPL has `137` regressions, including `19` `SplFileObject`, `29` `ArrayObject`, `9` `SplObjectStorage`, and `7` autoloading rows. All are absent in artifacts, so direct semantic responsibility is unproven until replay. | High after M0 replay |
| Reflection metadata expansion | `1f981cb8` reflection parameter metadata, `2d7a57e4` enum reflection metadata, `2498ca42` enum generated methods, `b180bf06` typed class-like constants, plus core class table additions for `ReflectionEnum*`. | Reflection has `110` regressions. Source changes are directly in the reflection surface, but all reflection regression rows are absent from candidate artifacts. | High after M0 replay |
| Broad builtin diagnostics and exception conversion | `fac5227e` throwable exception propagation and `ac0c0197` function argument diagnostics changed error-class/message mapping across builtins. | Explains preserved diagnostic-drift failures and can affect standard library rows if replay turns absences into failures. | Medium-high |
| Extension builtin/core-class additions | GMP, BcMath number edges, XMLReader factories, JSON throw, GD image diagnostics, POSIX metadata, password/bcrypt, crypt behavior, and core class table additions (`GMP`, `GdImage`, `JsonException`, `ReflectionEnum*`, `SplTempFileObject`). | Strong direct relevance to bcmath/xmlreader/posix/json/gd/password rows. Indirect risk to reflection/class metadata through core class table and internal function metadata. | Medium |
| Standard array/string runtime implementations | Diff scan found array-related changes only in reflection internal function metadata for `array_diff*` / `array_intersect*`; no first-order `array_*` or `str_replace` runtime implementation churn was visible in this range. | Weak direct source-diff explanation for the `249` standard array and `197` standard strings regressions. Those shards should be treated as replay/control-plane candidates first. | Medium M0, low direct source blame |

## Direct Failure Rows

Only `30 / 1166` regression rows have direct `FAILED`/`BORKED` status in the
candidate artifacts. Rows most aligned with the source diff are:

| Row | Status | Source-risk match |
| --- | --- | --- |
| `php-src/ext/standard/tests/directory/DirectoryClass_readonly_handle.phpt` | `FAILED` | Object property visibility/readonly diagnostics. |
| `php-src/ext/standard/tests/directory/DirectoryClass_readonly_path.phpt` | `FAILED` | Object property visibility/readonly diagnostics. |
| `php-src/ext/date/tests/DatePeriod_modify_readonly_property.phpt` | `FAILED` | Readonly property write/error surface. |
| `php-src/ext/date/tests/DatePeriod_properties2.phpt` | `FAILED` | Readonly/internal property metadata surface. |
| `php-src/ext/bcmath/tests/number/properties_unset.phpt` | `FAILED` | BcMath number property and object write/unset boundaries. |
| `php-src/ext/bcmath/tests/number/properties_write_error.phpt` | `FAILED` | BcMath number property and readonly/write diagnostics. |
| `php-src/ext/xmlreader/tests/014.phpt` | `FAILED` | XMLReader factory/method diagnostic changes. |

These are good narrow implementation/debug candidates because they have direct
candidate outcomes. They should not be used to explain the much larger absent
standard/SPL/reflection cliff without replay evidence.

## Commit Themes

Notable commits in the accepted-to-candidate range:

| Commit | Theme | Risk |
| --- | --- | --- |
| `88071e97` | Invalid property hooks | Parser/object diagnostic boundaries. |
| `fac5227e` | Throwable exception propagation | Error conversion and uncaught exception behavior. |
| `1f981cb8` | Reflection parameter metadata | Reflection API output and internal parameter defaults/types. |
| `48738670`, `af341085` | Property hook contracts/metadata | Property reflection, readonly/asymmetric diagnostics. |
| `4e26d128`, `75c29966` | Asymmetric property visibility parse/enforcement | Zend/property/readonly rows and class metadata. |
| `a1d98e14` | SPL memory file objects | SPL `SplFileObject`/`SplTempFileObject` rows. |
| `087e3ad3` | Class-order autoload variance | SPL autoloading and class dependency rows. |
| `2d7a57e4`, `2498ca42` | Enum reflection/generated methods | Reflection enum/class constant/method rows. |
| `ac0c0197` | Function argument diagnostics | Broad standard extension TypeError/ValueError message drift. |
| `ca69ead9` | XMLReader factories | XMLReader rows and reflection/internal class metadata. |
| `1b3c4f4d`, `99b183cc`, `5219685e`, `aaf4376f` | GMP support | GMP/BcMath/object comparison and core class metadata. |
| `6bca3d9c`, `eab7db21` | Password/html entity edges | Standard library builtins and new dependencies. |

## Recommended Next Actions

1. Replay a stratified accepted-vs-candidate sample before source repair:
   standard array/string rows from the selector reports, `SplFileObject` plus
   autoloading rows, and `ReflectionClass` plus `ReflectionParameter` rows.
2. Split direct failure work separately from absent-row work. The readonly
   property rows and `xmlreader/014.phpt` have preserved candidate failures and
   are suitable for narrow implementation/debug lanes.
3. Do not assign a broad standard array/string runtime rewrite from this diff.
   The source range does not show first-order array/string runtime churn matching
   the size of those absent regression shards.
4. If replay confirms SPL/reflection semantic failures, start with the source
   surfaces introduced by `a1d98e14`, `087e3ad3`, `1f981cb8`, `2d7a57e4`, and
   `b180bf06`.

## Commands Run

```sh
git log --oneline --no-merges 0b917f67..56fe9377
git diff --shortstat 0b917f67..56fe9377 --
git diff --stat --find-renames 0b917f67..56fe9377 --
git diff --name-status --find-renames 0b917f67..56fe9377 --
```

```sh
git diff --unified=0 0b917f67..56fe9377 -- compiler/src/interpreter.rs \
  | rg '^\\+\\s+(fn|pub fn) (call_|register_|execute_|resolve_|reflection_|spl_|json_|gmp_|gd_|xml|password)'
```

```sh
python3 - <<'PY'
from pathlib import Path
from collections import Counter

root = Path('/home/claude/supervised-php-compiler/state/logs/phpt-full-current-score-20260604T221205Z-php-src-f97ff59-public-56fe9377-source-56fe9377')
rows = [line.strip() for line in (root / 'regressions-from-latest-published-passes.txt').read_text().splitlines() if line.strip()]
status = {}
for line in (root / 'current-status.normalized.tsv').read_text().splitlines():
    state, row = line.split('\t', 1)
    status[row] = state

for prefix in [
    'php-src/ext/standard/tests/',
    'php-src/ext/spl/tests/',
    'php-src/ext/reflection/tests/',
    'php-src/ext/uri/tests/',
    'php-src/ext/posix/tests/',
    'php-src/ext/xmlreader/tests/',
]:
    subset = [row for row in rows if row.startswith(prefix)]
    print(prefix, len(subset), Counter(status.get(row, 'ABSENT') for row in subset))
PY
```

## Bottom Line

The accepted-to-candidate source diff contains real high-risk changes in
reflection, SPL, property visibility/readonly handling, core class metadata, and
builtin diagnostics. Those changes plausibly explain the small direct-failure
set and may explain replayed semantic failures. They do not, by themselves,
explain the dominant standard/SPL/reflection artifact absence pattern, so the
next deterministic step remains targeted accepted-vs-candidate replay rather
than a broad source repair lane.
