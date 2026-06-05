# Run 218 Command-Selection Recurrence

Developer: developer-446
Date: 2026-06-05
Scope: report-only triage artifact for `work_lanes#147` / `test_runs#218`

## Summary

`test_runs#218` is not evidence of a PHP compiler/runtime regression. It is a
fresh recurrence of the harness command-selection failure: the test loop ran
`python -m unittest discover -s tests -v` in a Rust/PHP repository and found
zero Python unittest tests.

The visible root harness import currently still selects the bad command even
though prior lane notes say a dry run returned `tools/run-tests.sh`.

## Evidence

SQLite `test_runs#218`:

```text
command: python -m unittest discover -s tests -v
status: failed
summary_json: {"error": 0, "failed": 1, "passed": 0, "skipped": 0}
full_log:
----------------------------------------------------------------------
Ran 0 tests in 0.000s

NO TESTS RAN
```

Local file facts:

```text
/home/claude/php-to-native-compiler/tools/run-tests.sh mode: 755
worktree tools/run-tests.sh mode: 755
```

Selector dry run from developer-446:

```text
PYTHONPATH=/home/claude/php-to-native-compiler/harness python3 - <<'PY'
from pathlib import Path
from llm_harness.testing_loop import discover_test_command
for p in [Path('/home/claude/php-to-native-compiler'), Path.cwd()]:
    print(p, discover_test_command(p))
PY
```

Output:

```text
/home/claude/php-to-native-compiler ['python', '-m', 'unittest', 'discover', '-s', 'tests', '-v']
/home/claude/php-to-native-compiler/.harness/worktrees/developer-446 ['python', '-m', 'unittest', 'discover', '-s', 'tests', '-v']
```

The imported function is:

```text
/home/claude/php-to-native-compiler/harness/llm_harness/testing_loop.py
```

and its current `discover_test_command()` checks `pytest`/`tests` before any
project-local `tools/run-tests.sh` preference.

## Deconflict State

developer-446 did not claim `work_lanes#147` because these agents already
recorded explicit claim/triage events:

```text
developer-434: lane_claimed work_lanes#147
developer-441: lane_claim work_lanes#147
developer-437: lane_claim work_lanes#147
developer-442: lane_triage / lane_triage_start
```

developer-446 recorded only a supporting triage/deconflict event and did not
edit harness code.

## Deterministic Next Action

The owning lane should patch the visible root harness selector so executable
`tools/run-tests.sh` is preferred before Python unittest/pytest fallback, then
prove the deployed artifact, not only a stale branch copy:

```text
PYTHONPATH=/home/claude/php-to-native-compiler/harness python3 - <<'PY'
from pathlib import Path
from llm_harness.testing_loop import discover_test_command
print(discover_test_command(Path('/home/claude/php-to-native-compiler')))
PY
```

Expected:

```text
['tools/run-tests.sh']
```

Focused verification should also rerun the existing root harness unittest:

```text
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v
```

Do not run the full project suite for this control-plane repair unless a
manager explicitly asks.
