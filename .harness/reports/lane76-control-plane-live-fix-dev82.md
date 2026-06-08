# Lane 76 Control-Plane Live Fix

Developer: developer-82
Lane: 76
Generated: 2026-06-07T17:12Z

Scope: harness/control-plane live patch only. No compiler/runtime/product
source files were edited. No full PHPT gate or `tools/run-tests.sh` suite was
run.

## Problem

Recent live `test_runs` repeatedly selected:

```text
python -m unittest discover -s tests -v
```

for `/home/claude/php-to-native-compiler`, then failed with:

```text
Ran 0 tests in 0.000s
NO TESTS RAN
```

This is a harness command-selection bug. The repository's product `tests/`
directory contains PHP fixtures, not Python unittest tests. The correct project
gate is the executable root script:

```text
tools/run-tests.sh
```

## Before

Selector dry-run against the live root zipapp:

```text
PYTHONPATH=/home/claude/php-to-native-compiler/harness python3 - <<'PY'
from pathlib import Path
from llm_harness.testing_loop import discover_test_command
print(discover_test_command(Path('/home/claude/php-to-native-compiler')))
PY
```

Output:

```text
['python', '-m', 'unittest', 'discover', '-s', 'tests', '-v']
```

Focused harness tests before patch:

```text
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v
```

Result:

```text
Ran 8 tests in 0.438s
FAILED (failures=4)
```

Failing tests were:

- `test_freeform_live_status_counts_as_running`
- `test_live_project_uses_project_run_tests_script`
- `test_liveness_keeps_freeform_active_rows`
- `test_test_loop_prefers_project_run_tests_script`

## Live Patch

Backed up the deployed zipapp first:

```text
/tmp/harness-dev82-lane76.orig
```

Patched `/home/claude/php-to-native-compiler/harness` embedded modules:

- `llm_harness/testing_loop.py`
  - `discover_test_command()` now prefers executable
    `tools/run-tests.sh` before generic Python discovery.
  - Python discovery is selected only when real Python test files or explicit
    Python test configuration exists.
  - zero-test unittest/pytest command failures are logged as
    `invalid_test_command` and do not create product failure worklanes.
- `llm_harness/db.py`
  - `list_agents(conn, "running")` now returns active non-terminal rows with
    no `ended_at`, so free-form working statuses count as live.
- `llm_harness/scheduler.py`
  - liveness checks use the active running predicate.
  - rows with missing tmux panes are still retired.
  - rows with no recorded tmux target remain eligible for idle prompting
    instead of being immediately marked crashed.

Updated the operational root harness test file:

```text
/home/claude/php-to-native-compiler/.harness/tests/test_codex_command.py
```

Added:

```text
test_zero_test_unittest_failure_does_not_create_product_lane
```

The new test uses an in-memory harness DB and an empty `tests/` directory to
prove a zero-test unittest command logs `invalid_test_command` without creating
`worklanes` or `bug_reports` rows.

## After

Selector dry-run:

```text
['tools/run-tests.sh']
```

Focused harness tests:

```text
test_freeform_live_status_counts_as_running ... ok
test_gpt55_uses_separate_reasoning_effort ... ok
test_live_project_uses_project_run_tests_script ... ok
test_liveness_ignores_ended_running_rows ... ok
test_liveness_keeps_freeform_active_rows ... ok
test_liveness_retires_missing_tmux_panes_before_alerting ... ok
test_safety_guard_requires_reasoning_effort ... ok
test_test_loop_prefers_project_run_tests_script ... ok
test_zero_test_unittest_failure_does_not_create_product_lane ... ok

Ran 9 tests in 0.533s
OK
```

Patched zipapp sha256:

```text
39aeba14dfda9b75dbfe7778c1f1d03ec6bf75f944032a8760649f1c7c77dd16
```

## Remaining Deployment Gap

The live root `harness` zipapp and
`/home/claude/php-to-native-compiler/.harness/tests/test_codex_command.py` are
not tracked by this worktree's Git index. The live deployment is fixed, but a
future rebuild or restart can regress unless the external authoritative
`llm_harness` source/build recipe is updated with the same changes.
