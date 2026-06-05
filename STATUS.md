# Harness Status

Last generated: 2026-06-05T23:42:56+00:00

## Goal
lThis project is a real PHP-to-native compiler effort in stable Rust. The target is full PHP compatibility as measured by the passed php tests "eval" and "variable variable" support is not required by nice to have. It can be the very last thing to try

## Metric
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)

## Agents
| name | role | current_status | tmux_window | worktree |
| --- | --- | --- | --- | --- |
| auditor-1 | Auditor | running | auditor-1 |  |
| coordinator-1 | Coordinator | running | coordinator-1 |  |
| developer-1 | Developer | running | developer-1 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-1 |
| developer-2 | Developer | running | developer-2 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-2 |
| integrator-1 | Integrator | running | integrator-1 |  |
| integrator-2 | Integrator | running | integrator-2 |  |
| integrator-3 | Integrator | running | integrator-3 |  |

## Work lanes
| id | title | role_type | card_type | stage | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1458 | Fix failing tests from run 1518 | Developer | implementation | planned | queued |  | 0.0 |
| 1457 | Fix failing tests from run 1517 | Developer | implementation | planned | queued |  | 0.0 |
| 1456 | Fix failing tests from run 1516 | Developer | implementation | planned | queued |  | 0.0 |
| 1455 | Fix failing tests from run 1515 | Developer | implementation | planned | queued |  | 0.0 |
| 1454 | Fix failing tests from run 1514 | Developer | implementation | planned | queued |  | 0.0 |
| 1453 | Fix failing tests from run 1513 | Developer | implementation | planned | queued |  | 0.0 |
| 1452 | Fix failing tests from run 1512 | Developer | implementation | planned | queued |  | 0.0 |
| 1451 | Fix failing tests from run 1511 | Developer | implementation | planned | queued |  | 0.0 |
| 1450 | Fix failing tests from run 1510 | Developer | implementation | planned | queued |  | 0.0 |
| 1449 | Fix failing tests from run 1509 | Developer | implementation | planned | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 2.34%, RAM 14.83%, disk free 9.2 GB.

## Recent events
- 2026-06-05T23:24:16+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-05T23:24:18+00:00 **worklane_created**: Fix failing tests from run 1515
- 2026-06-05T23:24:18+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:24:24+00:00 **worklane_created**: Fix failing tests from run 1516
- 2026-06-05T23:24:24+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:24:26+00:00 **scheduler**: Harness scheduler stopped by user
- 2026-06-05T23:24:29+00:00 **worklane_created**: Fix failing tests from run 1517
- 2026-06-05T23:24:29+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:24:34+00:00 **worklane_created**: Fix failing tests from run 1518
- 2026-06-05T23:24:34+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:41:12+00:00 **mcp**: Harness MCP server passed init preflight
- 2026-06-05T23:42:56+00:00 **index**: Indexed 1769 files during init

## Next steps
Move next worklane forward: Fix failing tests from run 1518
