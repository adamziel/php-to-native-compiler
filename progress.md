# Harness Status

Last generated: 2026-06-05T22:10:38+00:00

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

## Work lanes
| id | title | role_type | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- |
| 628 | Fix failing tests from run 688 | Developer | queued |  | 0.0 |
| 627 | Fix failing tests from run 687 | Developer | queued |  | 0.0 |
| 626 | Fix failing tests from run 686 | Developer | queued |  | 0.0 |
| 625 | Fix failing tests from run 685 | Developer | queued |  | 0.0 |
| 624 | Fix failing tests from run 684 | Developer | queued |  | 0.0 |
| 623 | Fix failing tests from run 683 | Developer | queued |  | 0.0 |
| 622 | Fix failing tests from run 682 | Developer | queued |  | 0.0 |
| 621 | Fix failing tests from run 681 | Developer | queued |  | 0.0 |
| 620 | Fix failing tests from run 680 | Developer | queued |  | 0.0 |
| 619 | Fix failing tests from run 679 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 8.75%, RAM 13.52%, disk free 8.66 GB.

## Recent events
- 2026-06-05T22:10:23+00:00 **worklane_status**: worklane#170 -> needs_verification
- 2026-06-05T22:10:23+00:00 **agent_report**: developer-1 reported needs_verification
- 2026-06-05T22:10:26+00:00 **worklane_created**: Fix failing tests from run 686
- 2026-06-05T22:10:26+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:10:28+00:00 **runtime_gate_failure_classified**: After harness selector repair, tools/run-tests.sh reaches real Rust gate. Reproduced parallel-only php_runtime --lib failure: 6 tests fail under default parallel cargo test; serial --test-threads=1 passes 420/420.
- 2026-06-05T22:10:30+00:00 **developer_capacity_available**: developer-1 completed lane 170 report_id 70 and is available for another narrow Developer lane.
- 2026-06-05T22:10:31+00:00 **worklane_created**: Fix failing tests from run 687
- 2026-06-05T22:10:31+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:10:37+00:00 **worklane_created**: Fix failing tests from run 688
- 2026-06-05T22:10:37+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:10:38+00:00 **poke**: Queued message for developer-2
- 2026-06-05T22:10:38+00:00 **poke_delivered**: Delivered poke to 1 agents

## Next steps
Move next worklane forward: Fix failing tests from run 688
