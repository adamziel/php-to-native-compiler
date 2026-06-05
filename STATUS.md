# Harness Status

Last generated: 2026-06-05T23:05:43+00:00

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
| 1246 | Fix failing tests from run 1306 | Developer | queued |  | 0.0 |
| 1245 | Fix failing tests from run 1305 | Developer | queued |  | 0.0 |
| 1244 | Fix failing tests from run 1304 | Developer | queued |  | 0.0 |
| 1243 | Fix failing tests from run 1303 | Developer | queued |  | 0.0 |
| 1242 | Fix failing tests from run 1302 | Developer | queued |  | 0.0 |
| 1241 | Fix failing tests from run 1301 | Developer | queued |  | 0.0 |
| 1240 | Fix failing tests from run 1300 | Developer | queued |  | 0.0 |
| 1239 | Fix failing tests from run 1299 | Developer | queued |  | 0.0 |
| 1238 | Fix failing tests from run 1298 | Developer | queued |  | 0.0 |
| 1237 | Fix failing tests from run 1297 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 2.62%, RAM 13.84%, disk free 9.25 GB.

## Recent events
- 2026-06-05T23:05:13+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:05:16+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-05T23:05:19+00:00 **worklane_created**: Fix failing tests from run 1302
- 2026-06-05T23:05:19+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:05:24+00:00 **worklane_created**: Fix failing tests from run 1303
- 2026-06-05T23:05:24+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:05:29+00:00 **worklane_created**: Fix failing tests from run 1304
- 2026-06-05T23:05:29+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:05:35+00:00 **worklane_created**: Fix failing tests from run 1305
- 2026-06-05T23:05:35+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T23:05:40+00:00 **worklane_created**: Fix failing tests from run 1306
- 2026-06-05T23:05:40+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes

## Next steps
Move next worklane forward: Fix failing tests from run 1306
