# Lane100 Proof Evaluator

Evaluator: developer-371
Timestamp: 2026-06-08T12:50:00+00:00
Lane evaluated: work_lanes#100
Evaluator lane: work_lanes#105

## Scope

Lane105 was requeued after developer-364 exited without a lane105 report. The
stored history shows this evaluator lane was folded into lane100 by
manager-14, and later coordinator/auditor events marked it stale unless it
records a final disposition.

This report evaluates the lane100 idle-alert liveness/dedupe proof only. It
does not edit compiler/runtime code, does not run a full PHPT gate, and does
not claim public score movement.

## Verdict

Lane100 has enough focused control-plane proof to close the stale evaluator
lane105 as completed/superseded by lane100.

The important caveat is still operational: the lane100 fix lives in the root
untracked harness zipapp and root `.harness/tests`, not in a tracked Rust/PHP
source commit in this worktree. That was already stated by developer-363 in
report 628 and remains the durable-source risk.

## Evidence

- work_lanes#100 is `status=integrated`, `stage=done`, and `done_at` is
  2026-06-08T11:05:41+00:00.
- developer-363 report 628 for card/worklane 100 states the implemented
  behavior:
  - `check_agent_liveness` skips ended agent rows before idle detection.
  - auditor targeting skips ended auditors.
  - high-churn idle and suspicious-sleep auditor alerts use stable source keys.
  - `repair_control_plane_cards` folds legacy per-agent idle-alert cards into
    one unresolved idle-alert card and marks older duplicates stale.
- developer-363 report 628 recorded passing proof:
  - focused lane100 regressions: 3/3 passed.
  - full `.harness` unittest slice: 24/24 passed.
  - `./harness --help` exited 0.
- I re-ran the live root proof from `/home/claude/php-to-native-compiler`:
  - `python3 -m unittest discover -s /home/claude/php-to-native-compiler/.harness/tests -v`
  - result: 24 tests ran in 2.145s, OK.
  - relevant passing tests included
    `test_liveness_retires_missing_tmux_panes_before_alerting`,
    `test_liveness_skips_ended_freeform_rows_before_idle_alerting`,
    `test_idle_alert_cards_share_one_unresolved_source_key`, and
    `test_repair_control_plane_dedupes_legacy_idle_alert_cards`.
- I also ran `./harness --help` from the root checkout; it exited 0 and showed
  the expected harness CLI commands.

## Candidate Counts

Historical pre-fix/storm evidence from event 93686:

- initial active rows: 281.
- initial idle over 30m: 170.
- post-cleanup active rows: 30.
- post-cleanup idle over 30m: 0.

Current live SQLite measurement using the harness `db.is_active_agent_status`
predicate at 2026-06-08T12:49:47+00:00:

- total agent rows: 713.
- active rows: 5.
- active rows idle over 30m: 0.
- active auditors idle over 30m: 0.
- active developers idle over 30m: 0.

This confirms the stale idle-alert candidate set is not currently rebuilding.

## Score Boundary

The accepted public PHPT score remains the only score to report:
7873 / 20294 = 38.79%.

The latest global `tools/run-tests.sh` rows are still failed known-red product
evidence and are not lane100 proof. They do not change the accepted public
score.

## Disposition

Lane105 should not be requeued to another Developer as active feature work.
The deterministic closure is:

- mark lane105 ready for review/completed as the lane100 proof evaluator
  disposition;
- keep lane100's durable-source caveat attached to the control-plane/harness
  maintenance backlog;
- assign developer capacity to a concrete non-overlapping product or
  integration lane after this report is accepted.
