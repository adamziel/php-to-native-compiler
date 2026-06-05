# Harness Status

Last generated: 2026-06-05T23:20:44+00:00

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
| id | title | role_type | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- |
| 1414 | Fix failing tests from run 1474 | Developer | queued |  | 0.0 |
| 1413 | Fix failing tests from run 1473 | Developer | queued |  | 0.0 |
| 1412 | Fix failing tests from run 1472 | Developer | queued |  | 0.0 |
| 1411 | Fix failing tests from run 1471 | Developer | queued |  | 0.0 |
| 1410 | Fix failing tests from run 1470 | Developer | queued |  | 0.0 |
| 1409 | Fix failing tests from run 1469 | Developer | queued |  | 0.0 |
| 1408 | Fix failing tests from run 1468 | Developer | queued |  | 0.0 |
| 1407 | Fix failing tests from run 1467 | Developer | queued |  | 0.0 |
| 1406 | Fix failing tests from run 1466 | Developer | queued |  | 0.0 |
| 1405 | Fix failing tests from run 1465 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 2.58%, RAM 14.83%, disk free 9.2 GB.

## Recent events
- 2026-06-05T23:20:12+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:20:18+00:00 **worklane_created**: Fix failing tests from run 1470
- 2026-06-05T23:20:18+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:20:23+00:00 **worklane_created**: Fix failing tests from run 1471
- 2026-06-05T23:20:23+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:20:28+00:00 **worklane_created**: Fix failing tests from run 1472
- 2026-06-05T23:20:28+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:20:34+00:00 **worklane_created**: Fix failing tests from run 1473
- 2026-06-05T23:20:34+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:20:39+00:00 **worklane_created**: Fix failing tests from run 1474
- 2026-06-05T23:20:39+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:20:40+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches

## Next steps
Move next worklane forward: Fix failing tests from run 1474
