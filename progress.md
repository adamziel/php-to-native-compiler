# Harness Status

Last generated: 2026-06-05T22:14:18+00:00

## Goal
lThis project is a real PHP-to-native compiler effort in stable Rust. The target is full PHP compatibility as measured by the passed php tests "eval" and "variable variable" support is not required by nice to have. It can be the very last thing to try

## Metric
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)

## Agents
| name | role | current_status | tmux_window | worktree |
| --- | --- | --- | --- | --- |
| coordinator-1 | Coordinator | running | coordinator-1 |  |
| developer-1 | Developer | running | developer-1 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-1 |
| developer-2 | Developer | running | developer-2 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-2 |
| integrator-1 | Integrator | running | integrator-1 |  |
| integrator-2 | Integrator | running | integrator-2 |  |

## Work lanes
| id | title | role_type | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- |
| 669 | Fix failing tests from run 729 | Developer | queued |  | 0.0 |
| 668 | Fix failing tests from run 728 | Developer | queued |  | 0.0 |
| 667 | Fix failing tests from run 727 | Developer | queued |  | 0.0 |
| 666 | Fix failing tests from run 726 | Developer | queued |  | 0.0 |
| 665 | Fix failing tests from run 725 | Developer | queued |  | 0.0 |
| 664 | Fix failing tests from run 724 | Developer | queued |  | 0.0 |
| 663 | Fix failing tests from run 723 | Developer | queued |  | 0.0 |
| 662 | Fix failing tests from run 722 | Developer | queued |  | 0.0 |
| 661 | Fix failing tests from run 721 | Developer | queued |  | 0.0 |
| 660 | Fix failing tests from run 720 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 4.87%, RAM 14.14%, disk free 8.47 GB.

## Recent events
- 2026-06-05T22:13:55+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:13:55+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-05T22:14:00+00:00 **worklane_created**: Fix failing tests from run 726
- 2026-06-05T22:14:00+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:14:05+00:00 **worklane_created**: Fix failing tests from run 727
- 2026-06-05T22:14:05+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:14:10+00:00 **worklane_created**: Fix failing tests from run 728
- 2026-06-05T22:14:10+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:14:16+00:00 **worklane_created**: Fix failing tests from run 729
- 2026-06-05T22:14:16+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:14:18+00:00 **poke**: Queued message for developer-1
- 2026-06-05T22:14:18+00:00 **poke_delivered**: Delivered poke to 1 agents

## Next steps
Move next worklane forward: Fix failing tests from run 729
