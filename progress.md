# Harness Status

Last generated: 2026-06-05T22:50:42+00:00

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
| 1078 | Fix failing tests from run 1138 | Developer | queued |  | 0.0 |
| 1077 | Fix failing tests from run 1137 | Developer | queued |  | 0.0 |
| 1076 | Fix failing tests from run 1136 | Developer | queued |  | 0.0 |
| 1075 | Fix failing tests from run 1135 | Developer | queued |  | 0.0 |
| 1074 | Fix failing tests from run 1134 | Developer | queued |  | 0.0 |
| 1073 | Fix failing tests from run 1133 | Developer | queued |  | 0.0 |
| 1072 | Fix failing tests from run 1132 | Developer | queued |  | 0.0 |
| 1071 | Fix failing tests from run 1131 | Developer | queued |  | 0.0 |
| 1070 | Fix failing tests from run 1130 | Developer | queued |  | 0.0 |
| 1069 | Fix failing tests from run 1129 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 3.66%, RAM 14.0%, disk free 9.25 GB.

## Recent events
- 2026-06-05T22:50:15+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:50:20+00:00 **worklane_created**: Fix failing tests from run 1134
- 2026-06-05T22:50:20+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:50:22+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-05T22:50:26+00:00 **worklane_created**: Fix failing tests from run 1135
- 2026-06-05T22:50:26+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:50:31+00:00 **worklane_created**: Fix failing tests from run 1136
- 2026-06-05T22:50:31+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:50:36+00:00 **worklane_created**: Fix failing tests from run 1137
- 2026-06-05T22:50:36+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:50:42+00:00 **worklane_created**: Fix failing tests from run 1138
- 2026-06-05T22:50:42+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes

## Next steps
Move next worklane forward: Fix failing tests from run 1138
