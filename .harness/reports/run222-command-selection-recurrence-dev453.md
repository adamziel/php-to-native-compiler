# Run222 Command-Selection Recurrence - developer-27

## Scope

- Worklane: `162`, "Harness command-selection recurrence after run222".
- Assigned agent: `developer-27`.
- Artifact path follows the stale lane acceptance text, which names
  `run222-command-selection-recurrence-dev453.md`; developer-453's authoritative
  lane was separately reconciled to worklane 160.
- Scope stayed harness/control-plane only. No compiler/runtime/product source
  files were edited.

## Current Verdict

The live recurrence was real at the start of this lane, but a concurrent harness
lane repaired the deployed zipapp before this lane wrote source changes.

Initial evidence from the live harness database during this lane:

- Recent rows through `test_runs#9498` selected
  `python -m unittest discover -s tests -v` at
  `c327b120925ac4afab3b4cfa051700d75634b807`.
- The deployed zipapp initially returned that same zero-test command from
  `llm_harness.testing_loop.discover_test_command(ROOT)`.
- Focused harness unittest failed 4/8:
  `test_freeform_live_status_counts_as_running`,
  `test_test_loop_prefers_project_run_tests_script`,
  `test_live_project_uses_project_run_tests_script`, and
  `test_liveness_keeps_freeform_active_rows`.

Concurrent repair evidence:

- Event `events#118975` records `developer-30` patching
  `/home/claude/php-to-native-compiler/harness`.
- The event reports focused `.harness` unittest passed and selector dry-run
  returned `['tools/run-tests.sh']`.

Current live state verified by developer-27:

- `discover_test_command(Path('/home/claude/php-to-native-compiler'))` returns
  `['tools/run-tests.sh']`.
- Focused harness unittest:
  `python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v`
  ran 8 tests and passed.
- Recent rows `test_runs#9518` through `test_runs#9529` selected
  `tools/run-tests.sh`, confirming the scheduler is no longer using the
  zero-test Python unittest fallback.

## Root Cause

The deployed harness zipapp's command selector regressed to a generic Python
test discovery path. In that state, a product `tests/` directory was enough to
select:

```text
python -m unittest discover -s tests -v
```

This repository's real full-suite gate is:

```text
tools/run-tests.sh
```

The same stale deployed artifact also used overly narrow agent liveness
predicates: `list_agents(conn, "running")` filtered exact
`current_status = "running"` instead of active non-terminal rows, and liveness
handling dropped free-form active statuses from idle alerting.

## Source And Deploy Path

The authoritative live import path is still the root zipapp:

```text
/home/claude/php-to-native-compiler/harness
```

No unpacked, tracked `llm_harness/testing_loop.py` source path was found in this
worktree. That means the operational patch is effective for the running
artifact, but restart durability still depends on applying the same fix to
whatever external source or generation step rebuilds `./harness`.

Patch points in the deployed zipapp:

- `llm_harness/testing_loop.py::discover_test_command()` must prefer the
  project runner before generic pytest/unittest discovery.
- `llm_harness/db.py::list_agents(conn, "running")` must return active
  non-terminal rows with `ended_at IS NULL`.
- `llm_harness/scheduler.py::HarnessScheduler.check_agent_liveness()` must use
  that active-row predicate and must not drop free-form active rows before idle
  prompting.

## Next Deterministic Action

No additional worklane 162 patch is needed while the current deployed zipapp
remains in place. The remaining deterministic hardening step is to make the
harness zipapp source durable: add or identify the tracked `llm_harness` source
tree/build recipe, apply the same selector/liveness changes there, and verify a
freshly rebuilt `./harness` still passes the focused `.harness` unittest and
selector dry-run.
