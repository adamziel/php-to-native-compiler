# Run28972 Command-Selection Live Fix

Developer: developer-94
Card: work_lanes#1776
Timestamp: 2026-06-07

## Finding

`test_runs#28972` is a recurrence of the harness command-selection failure,
not a PHP compiler/runtime regression. The failed command was:

```text
python -m unittest discover -s tests -v
```

The full log contained only:

```text
Ran 0 tests in 0.000s
NO TESTS RAN
```

The deployed root harness zipapp at
`/home/claude/php-to-native-compiler/harness` was stale again and selected the
zero-test Python unittest command for this Rust/PHP repository despite the
documented executable project gate at `tools/run-tests.sh`.

## Live Patch

Backed up the prior deployed zipapp to:

```text
/tmp/harness-dev94.orig
```

Patched and redeployed the root harness zipapp modules:

- `llm_harness/testing_loop.py`
  - `discover_test_command()` now prefers executable `tools/run-tests.sh`
    before Python discovery heuristics.
  - Python unittest/pytest runs that fail after exercising zero tests are
    recorded as `invalid_test_command` and do not create product failure lanes.
- `llm_harness/db.py`
  - `list_agents(conn, "running")` now returns non-ended agents whose statuses
    are active, including freeform working statuses.
- `llm_harness/scheduler.py`
  - `check_agent_liveness()` uses that active-row predicate and only retires
    agents when a recorded tmux target is missing.
  - Active rows without recorded tmux targets can still trigger idle alerts.

No compiler/runtime source files were edited.

## Verification

Selector dry run from the deployed root harness passed:

```text
python3 - <<'PY'
import sys
from pathlib import Path
ROOT = Path('/home/claude/php-to-native-compiler')
sys.path.insert(0, str(ROOT / 'harness'))
from llm_harness.testing_loop import discover_test_command
print(discover_test_command(ROOT))
PY
```

Output:

```text
['tools/run-tests.sh']
```

Focused harness tests passed:

```text
python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v
```

Result:

```text
Ran 9 tests in 0.474s
OK
```

The historical bad command still fails if run directly, as expected, because
the product `tests/` tree contains PHP fixtures rather than Python unittest
tests. The fix is that the live harness no longer selects that command for the
project gate.

## Artifact Hashes

```text
old: 970d50ad58c4175caf6567bccadc4a479fad6ad0a0282cf0dc9f9d9253403896
new: adec9d0168e49a375e8307487b5df2925c81db9b1b70e55980533b5a953a3ea6
```
