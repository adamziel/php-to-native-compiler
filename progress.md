# Harness Status

Last generated: 2026-06-05T22:35:41+00:00

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
| integrator-3 | Integrator | running | integrator-3 |  |

## Work lanes
| id | title | role_type | status | integration_queue | expected_metric_impact |
| --- | --- | --- | --- | --- | --- |
| 909 | Fix failing tests from run 969 | Developer | queued |  | 0.0 |
| 908 | Fix failing tests from run 968 | Developer | queued |  | 0.0 |
| 907 | Fix failing tests from run 967 | Developer | queued |  | 0.0 |
| 906 | Fix failing tests from run 966 | Developer | queued |  | 0.0 |
| 905 | Fix failing tests from run 965 | Developer | queued |  | 0.0 |
| 904 | Fix failing tests from run 964 | Developer | queued |  | 0.0 |
| 903 | Fix failing tests from run 963 | Developer | queued |  | 0.0 |
| 902 | Fix failing tests from run 962 | Developer | queued |  | 0.0 |
| 901 | Fix failing tests from run 961 | Developer | queued |  | 0.0 |
| 900 | Fix failing tests from run 960 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 2.72%, RAM 14.73%, disk free 8.69 GB.

## Recent events
- 2026-06-05T22:35:12+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:35:17+00:00 **worklane_created**: Fix failing tests from run 965
- 2026-06-05T22:35:17+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:35:22+00:00 **worklane_created**: Fix failing tests from run 966
- 2026-06-05T22:35:22+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:35:28+00:00 **worklane_created**: Fix failing tests from run 967
- 2026-06-05T22:35:28+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:35:29+00:00 **integration_idle**: No needs_verification or ready_for_integration lanes with branches
- 2026-06-05T22:35:33+00:00 **worklane_created**: Fix failing tests from run 968
- 2026-06-05T22:35:33+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:35:38+00:00 **worklane_created**: Fix failing tests from run 969
- 2026-06-05T22:35:38+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes

## Next steps
Move next worklane forward: Fix failing tests from run 969
