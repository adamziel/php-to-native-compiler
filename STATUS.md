# Harness Status

Last generated: 2026-06-06T08:17:32+00:00

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
| auditor-4 | Auditor | running | auditor-4 |  |
| conflict-resolver-1 | Conflict Resolver | crash | conflict-resolver-1 |  |
| conflict-resolver-2 | Conflict Resolver | crash | conflict-resolver-2 |  |
| coordinator-1 | Coordinator | stopped | coordinator-1 |  |
| coordinator-2 | Coordinator | crash | coordinator-2 |  |
| coordinator-3 | Coordinator | crash | coordinator-3 |  |
| coordinator-4 | Coordinator | stopped | coordinator-4 |  |
| coordinator-5 | Coordinator | running | coordinator-5 |  |
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
| developer-24 | Developer | running | developer-24 | /home/claude/php-to-native-compiler/.harness/worktrees/developer-24 |
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
| 1484 | Resolve integration failure for card #178: Fix failing tests from run 238 | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1483 | Respond to scheduler alert | Coordinator | control-plane | development | assigned |  | 0.0 |
| 1482 | Investigate scheduler alert | Auditor | integration-support | done | done |  | 0.0 |
| 1481 | Investigate scheduler alert | Auditor | integration-support | development | assigned |  | 0.0 |
| 1480 | Resolve integration failure for card #176: Fix failing tests from run 236 | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1479 | Resolve integration failure for card #175: Fix failing tests from run 235 | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1478 | Respond to scheduler alert | Coordinator | control-plane | done | done |  | 0.0 |
| 1477 | Resolve integration failure for card #174: Fix failing tests from run 234 | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1476 | Resolve integration failure for card #173: Fix failing tests from run 233 | Conflict Resolver | integration-support | planned | queued |  | 0.0 |
| 1475 | Resolve lane 172 runtime merge conflict only | Conflict Resolver | integration-support | done | done |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 5.42%, RAM 14.48%, disk free 5.16 GB.

## Recent events
- 2026-06-06T08:17:01+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T08:17:06+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 7218
- 2026-06-06T08:17:06+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T08:17:12+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 7219
- 2026-06-06T08:17:12+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T08:17:17+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 7220
- 2026-06-06T08:17:17+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T08:17:23+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 7221
- 2026-06-06T08:17:23+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-06T08:17:28+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-06T08:17:28+00:00 **worklane_deduplicated**: Updated existing failing-test card#1468 from run 7222
- 2026-06-06T08:17:28+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes

## Next steps
Move next worklane forward: Resolve integration failure for card #178: Fix failing tests from run 238
