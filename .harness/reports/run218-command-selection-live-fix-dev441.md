# Run 218 Command Selection Live Fix

Agent: developer-441
Lane: work_lanes#147
Test run: test_runs#218
Timestamp: 2026-06-05T19:08Z

## Finding

`test_runs#218` failed at commit `71479c716b10c526dcf2fc2a07dab2ef61d6b5ad`
because the live harness still selected:

```text
python -m unittest discover -s tests -v
```

The full log contained only:

```text
Ran 0 tests in 0.000s

NO TESTS RAN
```

This was a recurrence of the zero-test command-selection class. The live
zipapp at `/home/claude/php-to-native-compiler/harness` also failed the
focused root `.harness/tests` before patching:

- `test_test_loop_prefers_project_run_tests_script`
- `test_freeform_live_status_counts_as_running`
- `test_liveness_ignores_ended_running_rows`

## Live Patch

Backed up the previous deployed harness zipapp to:

```text
/tmp/harness-dev441.orig
```

Patched and redeployed only the live root harness zipapp modules:

- `llm_harness/testing_loop.py`
  - `discover_test_command()` now prefers executable `tools/run-tests.sh`
    before Python unittest/pytest heuristics.
- `llm_harness/db.py`
  - `list_agents(conn, "running")` now returns non-ended agents whose status is
    not a terminal lifecycle status, so freeform active statuses remain live.
- `llm_harness/scheduler.py`
  - `check_agent_liveness()` now excludes rows with `ended_at IS NOT NULL`
    before idle/suspicious alert routing.

No compiler/runtime/product source files were edited.

## Verification

Focused harness tests passed:

```text
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v
```

Result:

```text
Ran 7 tests in 0.223s

OK
```

Selector dry run passed:

```text
PYTHONPATH=/home/claude/php-to-native-compiler/harness python3 - <<'PY'
from pathlib import Path
from llm_harness.testing_loop import discover_test_command
root = Path('/home/claude/php-to-native-compiler')
print(discover_test_command(root))
PY
```

Result:

```text
['tools/run-tests.sh']
```

## Remaining Scope

The broader per-alert atomic auditor-spawn dedupe issue from
`bug_reports#3/#4/#5` remains separate unless another lane proves it. This
patch closes the run218 command-selection recurrence and the focused liveness
regressions covered by the existing root `.harness/tests`.
