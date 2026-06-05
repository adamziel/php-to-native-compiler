# Test Command Selection Recurrence: Run 63

Architect: architect-14
Timestamp: 2026-06-05T09:49:44Z
Scope: harness control plane only. No compiler/runtime source edits and no PHP
compatibility claim.

## Finding

`harness::test_command_selection::python_unittest_zero_tests_run63` is a
control-plane false failure. The harness test loop still selects:

```text
python -m unittest discover -s tests -v
```

for this Rust/PHP compiler repository. The top-level `tests/` directory is a
PHP fixture tree, not a Python unittest suite, so the command exits with:

```text
Ran 0 tests in 0.000s
NO TESTS RAN
```

The live selector confirms the stale behavior:

```text
discover_test_command(ROOT) -> ['python', '-m', 'unittest', 'discover', '-s', 'tests', '-v']
```

`tools/run-tests.sh` exists, is executable, and is the documented project gate:

```text
cargo test
cargo run -p phpc -- test
cargo run -p phpc -- test --compare-php
```

There are no `test*.py` files under the product `tests/` tree.

## Recurrence Evidence

The database shows the same bad command in runs `63`, `97`, `107`, `108`,
`110`, `111`, `127`, `164`, `195`, and `196`. Runs `195` and `196` occurred
after architect-13's report, proving the report alone did not move the live
selector. Their full logs contain only the zero-test unittest output.

`bug_reports#2` already tracks this failure class with the same root cause. The
failure keeps creating generic lanes such as `Fix failing tests from run 195`
and `Fix failing tests from run 196`, even though no product test has failed.

## Structural Root Cause

The harness does not have a repository-aware test contract. The current
`llm_harness.testing_loop.discover_test_command()` treats any `tests/`
directory as a Python test signal, then `summarize_results()` maps a nonzero
zero-test run to one generic failure. Because there are no parsed failing test
rows, `queue_test_fix_lane()` inserts an unkeyed "Fix failing tests from run N"
lane, and `maybe_invoke_architect()` queues repeated architect requests for the
same open bug without a spawn/request throttle.

This is a pipeline bug, not a compiler/runtime regression.

## Current Ownership Risk

The canonical lane is `work_lanes#8`, but its recorded owner
`developer-378` is stopped as of `2026-06-05T09:45:19+00:00`. The newer
zero-test lane `work_lanes#142` is in progress on `developer-391`, but it is
duplicate evidence for the same failure class. Manager should either reassign
lane 8 to a live owner or explicitly promote lane 142's owner to the canonical
command-selection owner. Do not let both lanes edit the harness zipapp.

## Reliability Refactor

1. Replace generic directory inference with explicit ordered project commands.
   Prefer executable `tools/run-tests.sh` before Python discovery. For Rust
   repositories without that script, prefer an explicit Cargo gate. Only select
   pytest/unittest when explicit Python test config or real Python test files
   are present.

2. Add a zero-test command guard. If pytest/unittest returns nonzero with no
   parsed tests and output says zero tests/no tests ran, classify it as
   `invalid_test_command`, not as a product suite failure. Record it under the
   harness command-selection bug and do not create ordinary Developer product
   fix lanes.

3. Dedupe queueing. `queue_test_fix_lane()` should key command-level failures
   by failure class and check for existing queued/in-progress canonical lanes
   before inserting another "Fix failing tests from run N" lane.

4. Dedupe architect escalation. `maybe_invoke_architect()` should check for an
   existing queued or started spawn request with the same repeated-bug title,
   or maintain a metadata throttle keyed by `test_nodeid`.

5. Keep acceptance focused. The control-plane acceptance suite is
   `.harness/tests`; the product full gate remains `tools/run-tests.sh`.

## Acceptance Checks

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

The selector check must print:

```text
['tools/run-tests.sh']
```

Scheduler-visible proof after the patch:

- no new `test_runs` row for this repo using
  `python -m unittest discover -s tests -v`
- no new duplicate zero-test "Fix failing tests from run N" lane
- no new duplicate Architect spawn for
  `harness::test_command_selection::python_unittest_zero_tests_run63` while a
  same-title request is already queued or started
