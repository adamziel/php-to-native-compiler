# Harness Status

Last generated: 2026-06-05T23:45:16+00:00

## Goal
lThis project is a real PHP-to-native compiler effort in stable Rust. The target is full PHP compatibility as measured by the passed php tests "eval" and "variable variable" support is not required by nice to have. It can be the very last thing to try

## Metric
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)

## Agents
| name | role | current_status | tmux_window | worktree |
| --- | --- | --- | --- | --- |
| auditor-1 | Auditor | stopped | auditor-1 |  |
| coordinator-1 | Coordinator | stopped | coordinator-1 |  |
| developer-1 | Developer | stopped | developer-1 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-1 |
| developer-2 | Developer | stopped | developer-2 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-2 |
| integrator-1 | Integrator | stopped | integrator-1 |  |
| integrator-2 | Integrator | stopped | integrator-2 |  |
| integrator-3 | Integrator | stopped | integrator-3 |  |

## Work lanes
| id | title | role_type | card_type | stage | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1462 | Respond to scheduler alert | Coordinator | control-plane | planned | queued |  | 0.0 |
| 1461 | Investigate scheduler alert | Auditor | integration-support | development | assigned |  | 0.0 |
| 1460 | Respond to scheduler alert | Coordinator | control-plane | development | assigned |  | 0.0 |
| 1459 | Fix failing tests from run 1519 | Developer | implementation | planned | queued |  | 0.0 |
| 1458 | Fix failing tests from run 1518 | Developer | implementation | planned | queued |  | 0.0 |
| 1457 | Fix failing tests from run 1517 | Developer | implementation | planned | queued |  | 0.0 |
| 1456 | Fix failing tests from run 1516 | Developer | implementation | planned | queued |  | 0.0 |
| 1455 | Fix failing tests from run 1515 | Developer | implementation | planned | queued |  | 0.0 |
| 1454 | Fix failing tests from run 1514 | Developer | implementation | planned | queued |  | 0.0 |
| 1453 | Fix failing tests from run 1513 | Developer | implementation | planned | queued |  | 0.0 |

## Tests
failed — python -m unittest discover -s tests -v (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 1.55%, RAM 13.66%, disk free 9.16 GB.

## Recent events
- 2026-06-05T23:44:52+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:44:58+00:00 **worklane_deduplicated**: Updated existing failing-test card#1459 from run 1542
- 2026-06-05T23:44:58+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:45:00+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-05T23:45:03+00:00 **worklane_deduplicated**: Updated existing failing-test card#1459 from run 1543
- 2026-06-05T23:45:03+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:45:08+00:00 **scheduler**: Harness scheduler stopped by user
- 2026-06-05T23:45:08+00:00 **worklane_deduplicated**: Updated existing failing-test card#1459 from run 1544
- 2026-06-05T23:45:08+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:45:10+00:00 **stop**: Stopped harness runtime
- 2026-06-05T23:45:16+00:00 **mcp**: Harness MCP tools exposed to Codex workers
- 2026-06-05T23:45:16+00:00 **tmux**: Support windows ready

## Next steps
Move next worklane forward: Respond to scheduler alert
