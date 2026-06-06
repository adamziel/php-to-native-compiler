# Harness Status

Last generated: 2026-06-06T11:58:26+00:00

## Goal
lThis project is a real PHP-to-native compiler effort in stable Rust. The target is full PHP compatibility as measured by the passed php tests "eval" and "variable variable" support is not required by nice to have. It can be the very last thing to try

## Metric
blocked_221205_candidate_phpt_passes: 7197.0 / 20294.0 (35.5%)

## Agents
| name | role | current_status | tmux_window | worktree |
| --- | --- | --- | --- | --- |
| auditor-1 | Auditor | stopped | auditor-1 |  |
| auditor-2 | Auditor | crash | auditor-2 |  |
| auditor-3 | Auditor | stopped | auditor-3 |  |
| auditor-4 | Auditor | stopped | auditor-4 |  |
| auditor-5 | Auditor | running | auditor-5 |  |
| conflict-resolver-1 | Conflict Resolver | crash | conflict-resolver-1 |  |
| conflict-resolver-2 | Conflict Resolver | crash | conflict-resolver-2 |  |
| conflict-resolver-3 | Conflict Resolver | running | conflict-resolver-3 |  |
| coordinator-1 | Coordinator | stopped | coordinator-1 |  |
| coordinator-2 | Coordinator | crash | coordinator-2 |  |
| coordinator-3 | Coordinator | crash | coordinator-3 |  |
| coordinator-4 | Coordinator | stopped | coordinator-4 |  |
| coordinator-5 | Coordinator | stopped | coordinator-5 |  |
| coordinator-6 | Coordinator | running | coordinator-6 |  |
| developer-1 | Developer | stopped | developer-1 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-1 |
| developer-10 | Developer | stopped | developer-10 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-10 |
| developer-11 | Developer | stopped | developer-11 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-11 |
| developer-12 | Developer | stopped | developer-12 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-12 |
| developer-13 | Developer | stopped | developer-13 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-13 |
| developer-14 | Developer | stopped | developer-14 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-14 |
| developer-15 | Developer | stopped | developer-15 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-15 |
| developer-16 | Developer | stopped | developer-16 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-16 |
| developer-17 | Developer | stopped | developer-17 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-17 |
| developer-18 | Developer | stopped | developer-18 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-18 |
| developer-19 | Developer | stopped | developer-19 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-19 |
| developer-2 | Developer | stopped | developer-2 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-2 |
| developer-20 | Developer | stopped | developer-20 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-20 |
| developer-21 | Developer | stopped | developer-21 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-21 |
| developer-22 | Developer | stopped | developer-22 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-22 |
| developer-23 | Developer | stopped | developer-23 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-23 |
| developer-24 | Developer | stopped | developer-24 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-24 |
| developer-25 | Developer | stopped | developer-25 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-25 |
| developer-26 | Developer | stopped | developer-26 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-26 |
| developer-27 | Developer | stopped | developer-27 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-27 |
| developer-28 | Developer | stopped | developer-28 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-28 |
| developer-29 | Developer | running | developer-29 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-29 |
| developer-3 | Developer | stopped | developer-3 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-3 |
| developer-30 | Developer | stopped | developer-30 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-30 |
| developer-31 | Developer | running | developer-31 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-31 |
| developer-32 | Developer | running | developer-32 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-32 |
| developer-33 | Developer | running | developer-33 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-33 |
| developer-34 | Developer | running | developer-34 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-34 |
| developer-35 | Developer | running | developer-35 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-35 |
| developer-4 | Developer | stopped | developer-4 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-4 |
| developer-5 | Developer | stopped | developer-5 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-5 |
| developer-6 | Developer | stopped | developer-6 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-6 |
| developer-7 | Developer | stopped | developer-7 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-7 |
| developer-8 | Developer | stopped | developer-8 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-8 |
| developer-9 | Developer | stopped | developer-9 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-9 |
| integrator-1 | Integrator | stopped | integrator-1 |  |
| integrator-2 | Integrator | stopped | integrator-2 |  |
| integrator-3 | Integrator | stopped | integrator-3 |  |
| integrator-4 | Integrator | running | integrator-4 |  |

## Work lanes
| id | title | role_type | card_type | stage | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1529 | Provide next sanctioned Developer lane after developer-35 capacity check | Developer | implementation | planned | queued |  | 0.0 |
| 1528 | Respond to scheduler alert | Coordinator | control-plane | planned | queued |  | 0.0 |
| 1527 | Resolve integration failure for card #1486: Fix global test suite failures | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1526 | Provide next sanctioned Developer lane after developer-34 capacity check | Developer | implementation | planned | queued |  | 0.0 |
| 1525 | Assign fresh sanctioned Developer card after superseded lane 10 | Developer | implementation | planned | queued |  | 0.0 |
| 1524 | Provide next narrow Developer lane after developer-33 capacity check | Developer | implementation | planned | queued |  | 0.0 |
| 1523 | Fix global test suite failures | Developer | implementation | done | stale |  | 0.0 |
| 1522 | Fix global test suite failures | Developer | implementation | done | stale |  | 0.0 |
| 1521 | Fix global test suite failures | Developer | implementation | done | stale |  | 0.0 |
| 1520 | Fix global test suite failures | Developer | implementation | done | stale |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 2.12%, RAM 16.01%, disk free 21.68 GB.

## Recent events
- 2026-06-06T11:57:54+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T11:57:59+00:00 **worklane_deduplicated**: Updated existing failing-test card#1486 from run 9641
- 2026-06-06T11:57:59+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T11:58:01+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-06T11:58:05+00:00 **worklane_deduplicated**: Updated existing failing-test card#1486 from run 9642
- 2026-06-06T11:58:05+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T11:58:10+00:00 **worklane_deduplicated**: Updated existing failing-test card#1486 from run 9643
- 2026-06-06T11:58:10+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T11:58:16+00:00 **worklane_deduplicated**: Updated existing failing-test card#1486 from run 9644
- 2026-06-06T11:58:16+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T11:58:21+00:00 **worklane_deduplicated**: Updated existing failing-test card#1486 from run 9645
- 2026-06-06T11:58:21+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes

## Next steps
Move next worklane forward: Provide next sanctioned Developer lane after developer-35 capacity check
