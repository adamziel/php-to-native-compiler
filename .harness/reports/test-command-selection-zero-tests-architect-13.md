# Test Command Selection Zero-Tests Root Cause

Architect: architect-13
Timestamp: 2026-06-05T09:33:23Z
Scope: harness control plane only. No compiler/runtime source edits and no
PHP compatibility claim.

## Finding

`harness::test_command_selection::python_unittest_zero_tests_run63` is a
recurring false failure caused by the harness selecting a Python unittest
command for a Rust/PHP repository whose top-level `tests/` directory contains
PHP fixture data, not Python tests.

The imported harness zipapp currently implements
`llm_harness.testing_loop.discover_test_command(root)` as:

- if `pytest.ini`, `pyproject.toml`, or `tests/` exists and `pytest` is
  importable, use `python -m pytest -vv`
- else if `tests/` exists, use
  `python -m unittest discover -s tests -v`
- else use generic `python -m unittest discover`

It does not check the documented project gate `tools/run-tests.sh`, even though
`tools/run-tests.sh` exists and is executable in this repository.

## Evidence

- Direct reproduction from the repository root:
  `python3 -m unittest discover -s tests -v` exits `5` with
  `Ran 0 tests` and `NO TESTS RAN`.
- The only Python harness self-test file is
  `.harness/tests/test_codex_command.py`; there are no `test*.py` files under
  the product `tests/` fixture tree.
- Focused harness self-tests currently fail:
  `python3 -m unittest discover -s .harness/tests -v` reports two failures:
  `test_test_loop_prefers_project_run_tests_script` and
  `test_freeform_live_status_counts_as_running`.
- The DB shows repeated stale-command runs using the same bad selector:
  test runs `63`, `97`, `107`, `127`, and `164` all recorded
  `python -m unittest discover -s tests -v`.
- `bug_reports#2` has five occurrences and already identifies the command as
  a zero-test false failure that stalls the scheduler.
- The current canonical implementation lane for this slice is lane `8`, but
  its recorded owner `developer-277` is stopped. That explains why run `164`
  recurred after earlier duplicate-lane classification.

## Structural Root Cause

The scheduler has no repository-aware test-command contract. It infers a full
suite from generic file/directory names, treats `tests/` as a Python signal,
and does not require proof that the selected command exercises meaningful
project tests. The false zero-test command then flows through the normal
failed-suite machinery:

- `summarize_results()` turns a command failure with no parsed tests into one
  generic failure.
- `queue_test_fix_lane()` creates generic "Fix failing tests from run N" lanes
  with no stable product test id.
- `maybe_invoke_architect()` queues repeated Architect requests for the same
  open repeated bug without a title/key throttle.

This creates duplicate work lanes and repeated architect escalations while no
compiler/runtime behavior has changed.

## Reliability Refactor Plan

1. Make test command selection explicit and ordered.
   - Prefer executable `tools/run-tests.sh` before any generic Python discovery.
   - For Rust repositories without `tools/run-tests.sh`, prefer a documented
     Cargo command over Python.
   - Only select pytest/unittest when actual Python test files or explicit
     Python test config exist. A directory named `tests/` is not sufficient.

2. Add a zero-test guard.
   - Classify `pytest`/`unittest` commands that run zero tests as
     `invalid_test_command`, not as product suite failures.
   - Do not create ordinary Developer fix lanes for zero-test command failures.
   - Route the event to the harness/control-plane bug report and one canonical
     command-selection lane.

3. Dedupe generated work.
   - `queue_test_fix_lane()` should check for an existing open queued or
     in-progress lane keyed by the same failure class before inserting.
   - `maybe_invoke_architect()` should check for an existing queued or started
     spawn request with the same repeated-bug title, or use a metadata throttle
     keyed by `test_nodeid`.

4. Keep harness self-tests in the harness lane.
   - `.harness/tests/test_codex_command.py` is the correct focused acceptance
     suite for this control-plane behavior.
   - The product test loop should not select `.harness/tests` as the project
     full gate; it should use `tools/run-tests.sh`.

5. Make deployment verifiable.
   - Patch the source of the harness zipapp or its build input, not only a
     worker-local checkout artifact.
   - Record a scheduler-visible dry-run proof that
     `discover_test_command(ROOT)` returns `["tools/run-tests.sh"]`.
   - Run `python3 -m unittest discover -s .harness/tests -v` after the patch.

## Acceptance Checks

Required focused checks for lane 8:

```sh
python3 -m unittest discover -s .harness/tests -v
python3 - <<'PY'
import sys
from pathlib import Path
ROOT = Path.cwd()
sys.path.insert(0, str(ROOT / "harness"))
from llm_harness.testing_loop import discover_test_command
print(discover_test_command(ROOT))
PY
```

The second command must print `['tools/run-tests.sh']`.

Required scheduler-visible proof:

- no new `test_runs` row using `python -m unittest discover -s tests -v`
  for this repository after the fix
- no duplicate zero-test "Fix failing tests from run N" lane for the same
  failure class
- no duplicate Architect spawn request for the same repeated bug while one is
  already queued or started

## Scheduling Note

Lane `8` should be requeued or reassigned before another test-loop tick. The
current DB state still marks lane `8` `in_progress`, but its owner
`developer-277` is stopped and its worktree does not contain a visible harness
zipapp candidate. Leaving that lane in progress will keep the command-selection
repair looking owned while the root harness remains stale.
