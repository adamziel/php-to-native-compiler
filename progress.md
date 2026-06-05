# Harness Status

Last generated: 2026-06-05T22:05:40+00:00

## Goal
lThis project is a real PHP-to-native compiler effort in stable Rust. The target is full PHP compatibility as measured by the passed php tests "eval" and "variable variable" support is not required by nice to have. It can be the very last thing to try

## Metric
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)

## Agents
None.

## Work lanes
| id | title | role_type | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- |
| 573 | Fix failing tests from run 633 | Developer | queued |  | 0.0 |
| 572 | Fix failing tests from run 632 | Developer | queued |  | 0.0 |
| 571 | Fix failing tests from run 631 | Developer | queued |  | 0.0 |
| 570 | Fix failing tests from run 630 | Developer | queued |  | 0.0 |
| 569 | Fix failing tests from run 629 | Developer | queued |  | 0.0 |
| 568 | Fix failing tests from run 628 | Developer | queued |  | 0.0 |
| 567 | Fix failing tests from run 627 | Developer | queued |  | 0.0 |
| 566 | Fix failing tests from run 626 | Developer | queued |  | 0.0 |
| 565 | Fix failing tests from run 625 | Developer | queued |  | 0.0 |
| 564 | Fix failing tests from run 624 | Developer | queued |  | 0.0 |

## Tests
No test runs recorded yet.

## Resource samples
Latest: CPU 9.67%, RAM 13.76%, disk free 9.54 GB.

## Recent events
- 2026-06-05T22:04:58+00:00 **worklane_created**: Fix failing tests from run 633
- 2026-06-05T22:04:58+00:00 **spawn_request**: test-loop requested Architect: Find systemic cause for repeated failure: harness/idle-alert-ended-agents
- 2026-06-05T22:04:58+00:00 **spawn_request**: test-loop requested Architect: Find systemic cause for repeated failure: harness/idle-alert-missing-window-undelivered-assignment
- 2026-06-05T22:04:58+00:00 **spawn_request**: test-loop requested Architect: Find systemic cause for repeated failure: harness/idle-alert-auditor-spawn-storm
- 2026-06-05T22:04:58+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:05:00+00:00 **worklane_status**: worklane#168 -> control_plane_verified_live_non_durable
- 2026-06-05T22:05:00+00:00 **agent_report**: integrator-48 reported control_plane_verified_live_non_durable
- 2026-06-05T22:05:00+00:00 **integration_addendum**: Lane168 final re-probe found concurrent live harness fix now effective: selector tools/run-tests.sh and .harness 8/8; prior stale evidence retained as durability warning.
- 2026-06-05T22:05:00+00:00 **stop**: Stopped harness runtime
- 2026-06-05T22:05:00+00:00 **reset_counters**: Reset stale harness telemetry after upgrade
- 2026-06-05T22:05:40+00:00 **mcp**: Harness MCP tools exposed to Codex workers
- 2026-06-05T22:05:40+00:00 **tmux**: Support windows ready

## Next steps
Move next worklane forward: Fix failing tests from run 633
