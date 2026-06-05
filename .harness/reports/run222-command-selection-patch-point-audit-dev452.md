# Run222 Command-Selection Patch-Point Audit

Developer: developer-452
Work lane: work_lanes#159
Scope: read-only audit; no compiler/runtime edits, no full suite, no public score movement.

## Verdict

The SQLite work_lanes row is authoritative: this is lane 159, not lane 149.

`test_runs#222` is a real recurrence of the harness command-selection bug, not
a PHP compiler/runtime failure. At 2026-06-05T19:48:39+00:00 the test loop ran:

```text
python -m unittest discover -s tests -v
```

and recorded only:

```text
Ran 0 tests in 0.000s
NO TESTS RAN
```

The live deployed harness code at the start of this audit explained that result:
`llm_harness/testing_loop.py::discover_test_command()` selected Python unittest
from the mere presence of `tests/` before considering the project gate
`tools/run-tests.sh`.

During this audit, lane158 activity partially changed the live zipapp at
`/home/claude/php-to-native-compiler/harness` around mtime
`2026-06-05 21:53:20 +0200`: the selector now returns `['tools/run-tests.sh']`.
The focused harness tests still fail 2/8 because the liveness/source-status
half remains stale in the same deployed zipapp.

## Authoritative Source vs Deployed Artifact

Checked root path:

```text
/home/claude/php-to-native-compiler
```

Deployed harness path:

```text
/home/claude/php-to-native-compiler/harness
```

That file is a Python zipapp:

```text
shebang: #!/usr/bin/env python3
zip signature offset: 23
embedded modules include llm_harness/testing_loop.py, llm_harness/db.py,
llm_harness/scheduler.py
```

Current deployed zipapp metadata observed after the concurrent lane158 partial
patch:

```text
size: 159585
mode: 755
mtime: 2026-06-05 21:53:20.628317631 +0200
sha256: 8849126a6437602adea1db8d99295c610bf37df070f6d61ddbeee408f94d4596
```

No versioned authoritative Python source tree for `llm_harness` is present in
the root checkout when excluding `.harness/worktrees`, `.git`, and `target`.
`git ls-files` does not list root `harness` or
`.harness/tests/test_codex_command.py`; prior run218 repair commits
`d948785f` and `731ea3ff` committed report files only. That is the durability
gap: previous fixes mutated or verified the deployed zipapp, but the deployed
code is not backed here by a tracked source/deploy path, so a restart/rebuild can
restore stale zipapp contents.

## Why Run222 Selected Python Unittest

SQLite evidence:

| Run | Time | Command | Result |
| --- | --- | --- | --- |
| test_runs#220 | 2026-06-05T19:20:36Z | `tools/run-tests.sh` | real php_runtime failure |
| test_runs#221 | 2026-06-05T19:36:25Z | `tools/run-tests.sh` | real php_runtime failure |
| test_runs#222 | 2026-06-05T19:48:39Z | `python -m unittest discover -s tests -v` | zero tests |

Pre-patch live selector code in embedded
`llm_harness/testing_loop.py` had this ordering:

```text
wants_pytest = pytest.ini or pyproject.toml or tests/
if wants_pytest and pytest is importable: python -m pytest -vv
if tests/ exists: python -m unittest discover -s tests -v
fallback: python -m unittest discover
```

Because this repository has a product `tests/` directory but no Python unittest
suite there, the test loop selected a command that runs zero tests. The correct
project gate is the executable root script:

```text
tools/run-tests.sh
```

Current state after concurrent lane158 partial patch:

```text
discover_test_command(Path('/home/claude/php-to-native-compiler'))
=> ['tools/run-tests.sh']
```

The selector recurrence itself appears partially repaired in the live zipapp as
of this report, but only in the deployed artifact and not in a tracked source
path.

## Why .harness Tests Fail

At the beginning of this audit, the focused root harness tests failed 4/8:

```text
test_freeform_live_status_counts_as_running ... FAIL
test_live_project_uses_project_run_tests_script ... FAIL
test_liveness_ignores_ended_running_rows ... FAIL
test_test_loop_prefers_project_run_tests_script ... FAIL
```

After the concurrent lane158 selector-only zipapp change, the same focused test
command still fails 2/8:

```text
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v

Ran 8 tests in 0.369s
FAILED (failures=2)
```

Remaining failures:

```text
test_freeform_live_status_counts_as_running
  db.list_agents(conn, "running") returns [] for current_status
  "inspecting PHPT lane" because embedded db.py still filters exact
  current_status = "running".

test_liveness_ignores_ended_running_rows
  scheduler.check_agent_liveness() still includes rows with ended_at set when
  current_status remains "running", so it prompts an auditor for
  developer-ended.
```

Current stale deployed code:

```text
llm_harness/db.py:688-693
def list_agents(conn, status=None):
    if status is None:
        return list(conn.execute("SELECT * FROM agents ORDER BY role, name"))
    return list(conn.execute(
        "SELECT * FROM agents WHERE current_status = ? ORDER BY role, name",
        (status,),
    ))

llm_harness/scheduler.py:761-765
agents = [
    agent
    for agent in db.list_agents(conn)
    if db.is_active_agent_status(agent["current_status"])
]
```

## Exact Patch/Deploy Point for Lane158

Patch the deployed root harness zipapp, not a worktree copy, and verify imports
from that zipapp:

```text
/home/claude/php-to-native-compiler/harness
```

Embedded modules to patch:

1. `llm_harness/testing_loop.py::discover_test_command()`
   - Keep the current selector repair, but make the project runner check the
     intended executable gate explicitly:

```text
project_runner = root_path / "tools" / "run-tests.sh"
if project_runner.is_file() and project_runner.stat().st_mode & 0o111:
    return ["tools/run-tests.sh"]
```

2. `llm_harness/db.py::list_agents()`
   - For `status == "running"`, return non-ended agents whose status is not in
     `AGENT_TERMINAL_STATUSES`, instead of exact `current_status = "running"`.
   - Keep exact status filtering for non-running statuses if callers rely on it.

3. `llm_harness/scheduler.py::check_agent_liveness()`
   - Exclude rows with `ended_at IS NOT NULL` before idle/suspicious routing.
   - Prefer sharing the same active-row predicate used by `db.list_agents()`
     rather than duplicating lifecycle logic.

Deploy step:

- Extract or rewrite the zipapp at `/home/claude/php-to-native-compiler/harness`
  with the patched embedded modules, preserving the `#!/usr/bin/env python3`
  shebang and executable mode.
- Back up the prior artifact first, as prior lanes did with `/tmp/harness-*.orig`.
- Do not verify against a branch-local or temporary extracted module alone.

Minimum verification:

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

Then:

```text
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v
```

Expected after the liveness patch:

```text
Ran 8 tests
OK
```

No full `tools/run-tests.sh` gate is needed for this control-plane patch-point
audit unless a coordinator explicitly requests it.

## Next Deterministic Action

Lane158 should finish the zipapp liveness repair in `db.py` and `scheduler.py`,
then rerun the two focused checks above from the root deployed zipapp path. The
longer-term durability fix is to make the harness Python source and zipapp build
path versioned; otherwise future restarts can repeat the same stale-artifact
recurrence even after live zipapp proofs pass.
