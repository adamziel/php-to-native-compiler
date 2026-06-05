# Run 218 Selector Recurrence Evidence

Developer: developer-437
Date: 2026-06-05
Scope: read-only support evidence for work_lanes#147 / test_run#218.

## Summary

test_run#218 is not a product test failure. It is another harness
command-selection recurrence:

- Command recorded by the harness: `python -m unittest discover -s tests -v`
- Full log: `Ran 0 tests` / `NO TESTS RAN`
- Commit: `71479c716b10c526dcf2fc2a07dab2ef61d6b5ad`

developer-437 is not taking code ownership for work_lanes#147 because earlier
claim events exist for developer-434 and developer-441, and subsequent
implementation-start events exist for developer-436/developer-442. This report
captures deterministic evidence only.

## Reproduction

Command:

```sh
PYTHONPATH=/home/claude/php-to-native-compiler/harness python3 - <<'PY'
from llm_harness.testing_loop import discover_test_command
for root in ['/home/claude/php-to-native-compiler',
             '/home/claude/php-to-native-compiler/.harness/worktrees/developer-437']:
    print(root)
    print(discover_test_command(root))
PY
```

Observed result:

```text
/home/claude/php-to-native-compiler
['python', '-m', 'unittest', 'discover', '-s', 'tests', '-v']
/home/claude/php-to-native-compiler/.harness/worktrees/developer-437
['python', '-m', 'unittest', 'discover', '-s', 'tests', '-v']
```

Focused harness test command:

```sh
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v
```

Observed result: 7 tests ran, 3 failed:

- `test_test_loop_prefers_project_run_tests_script`
- `test_freeform_live_status_counts_as_running`
- `test_liveness_ignores_ended_running_rows`

## Deployed Zipapp Gaps

The loaded harness module is the root zipapp:

- `llm_harness.testing_loop`: `/home/claude/php-to-native-compiler/harness/llm_harness/testing_loop.py`
- `llm_harness.db`: `/home/claude/php-to-native-compiler/harness/llm_harness/db.py`
- `llm_harness.scheduler`: `/home/claude/php-to-native-compiler/harness/llm_harness/scheduler.py`

Inspection through Python `zipfile` shows the deployed zipapp lacks the prior
control-plane fixes:

- `testing_loop.discover_test_command()` checks `pytest.ini`,
  `pyproject.toml`, or `tests/` before considering `tools/run-tests.sh`, so
  this Rust/PHP project is still routed to Python unittest.
- `db.list_agents(conn, "running")` performs exact
  `current_status = 'running'` matching, so free-form active statuses are not
  included.
- `scheduler.check_agent_liveness()` filters by active status but does not
  exclude rows with `ended_at` set, so ended rows can still alert as idle.

## Recommended Fix Boundary

Keep the active fix limited to the deployed harness artifact and focused
`.harness/tests`:

- Prefer executable `tools/run-tests.sh` before Python test discovery.
- Make `list_agents(conn, "running")` use the active-agent predicate and
  ignore rows with `ended_at IS NOT NULL`.
- Make scheduler liveness skip or retire rows with `ended_at` before prompting.

No compiler/runtime/product source changes are implicated by this recurrence.
