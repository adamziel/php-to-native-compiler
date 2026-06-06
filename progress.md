# Harness Status

Last generated: 2026-06-06T00:30:18+00:00

## Goal
lThis project is a real PHP-to-native compiler effort in stable Rust. The target is full PHP compatibility as measured by the passed php tests "eval" and "variable variable" support is not required by nice to have. It can be the very last thing to try

## Metric
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)

## Agents
| name | role | current_status | tmux_window | worktree |
| --- | --- | --- | --- | --- |
| auditor-1 | Auditor | stopped | auditor-1 |  |
| auditor-2 | Auditor | crash | auditor-2 |  |
| conflict-resolver-1 | Conflict Resolver | crash | conflict-resolver-1 |  |
| coordinator-1 | Coordinator | stopped | coordinator-1 |  |
| coordinator-2 | Coordinator | crash | coordinator-2 |  |
| coordinator-3 | Coordinator | running | coordinator-3 |  |
| developer-1 | Developer | stopped | developer-1 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-1 |
| developer-10 | Developer | stopped | developer-10 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-10 |
| developer-11 | Developer | stopped | developer-11 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-11 |
| developer-12 | Developer | stopped | developer-12 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-12 |
| developer-13 | Developer | stopped | developer-13 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-13 |
| developer-14 | Developer | stopped | developer-14 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-14 |
| developer-15 | Developer | stopped | developer-15 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-15 |
| developer-16 | Developer | stopped | developer-16 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-16 |
| developer-17 | Developer | running | developer-17 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-17 |
| developer-2 | Developer | stopped | developer-2 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-2 |
| developer-3 | Developer | stopped | developer-3 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-3 |
| developer-4 | Developer | stopped | developer-4 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-4 |
| developer-5 | Developer | stopped | developer-5 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-5 |
| developer-6 | Developer | stopped | developer-6 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-6 |
| developer-7 | Developer | stopped | developer-7 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-7 |
| developer-8 | Developer | stopped | developer-8 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-8 |
| developer-9 | Developer | stopped | developer-9 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-9 |
| integrator-1 | Integrator | stopped | integrator-1 |  |
| integrator-2 | Integrator | stopped | integrator-2 |  |
| integrator-3 | Integrator | stopped | integrator-3 |  |

## Work lanes
| id | title | role_type | card_type | stage | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1474 | Resolve integration failure for card #172: Fix failing tests from run 232 | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1473 | Respond to scheduler alert | Coordinator | control-plane | planned | queued |  | 0.0 |
| 1472 | Resolve current integration failures without feature work | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1471 | Investigate scheduler alert | Auditor | integration-support | done | done |  | 0.0 |
| 1470 | Resolve integration failure for card #166: Fix failing tests from run 226 | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1469 | Maintain Developer capacity | Developer | implementation | done | integrated | ready_fast_path | 0.0 |
| 1468 | Fix failing tests from run 1603 | Developer | implementation | planned | queued |  | 0.0 |
| 1467 | Resolve integration failure for card #1464: Maintain Developer capacity | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1466 | Repair live harness command selector recurrence | Developer | implementation | done | integrated | ready_fast_path | 0.0 |
| 1465 | Resolve integration failure for card #1463: Maintain Developer capacity | Conflict Resolver | integration-support | planned | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 17.76%, RAM 14.04%, disk free 7.83 GB.

## Recent events
- 2026-06-06T00:29:53+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T00:29:53+00:00 **uncarded_agent_stopped**: Stopped developer-16 because it had 0 development cards
- 2026-06-06T00:29:58+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 2044
- 2026-06-06T00:29:58+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T00:30:03+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 2045
- 2026-06-06T00:30:03+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T00:30:09+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 2046
- 2026-06-06T00:30:09+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T00:30:12+00:00 **developer_context_loaded**: Loaded required startup docs from worktree; DEVELOPMENT.md missing in worktree root, checking repository root and harness assignments.
- 2026-06-06T00:30:14+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-06T00:30:14+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 2047
- 2026-06-06T00:30:14+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes

## Next steps
Move next worklane forward: Resolve integration failure for card #172: Fix failing tests from run 232
