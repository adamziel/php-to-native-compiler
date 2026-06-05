# Harness Status

Last generated: 2026-06-05T22:16:44+00:00

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
| 696 | Fix failing tests from run 756 | Developer | queued |  | 0.0 |
| 695 | Fix failing tests from run 755 | Developer | queued |  | 0.0 |
| 694 | Fix failing tests from run 754 | Developer | queued |  | 0.0 |
| 693 | Fix failing tests from run 753 | Developer | queued |  | 0.0 |
| 692 | Fix failing tests from run 752 | Developer | queued |  | 0.0 |
| 691 | Fix failing tests from run 751 | Developer | queued |  | 0.0 |
| 690 | Fix failing tests from run 750 | Developer | queued |  | 0.0 |
| 689 | Fix failing tests from run 749 | Developer | queued |  | 0.0 |
| 688 | Fix failing tests from run 748 | Developer | queued |  | 0.0 |
| 687 | Fix failing tests from run 747 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 4.92%, RAM 13.37%, disk free 7.91 GB.

## Recent events
- 2026-06-05T22:16:35+00:00 **worklane_created**: Fix failing tests from run 755
- 2026-06-05T22:16:35+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:16:37+00:00 **worklane_status**: worklane#170 -> superseded_by_lane171_live_control_plane
- 2026-06-05T22:16:37+00:00 **agent_report**: integrator-2 reported superseded_by_lane171_live_control_plane
- 2026-06-05T22:16:38+00:00 **worklane_status**: worklane#171 -> control_plane_verified_live_non_durable
- 2026-06-05T22:16:38+00:00 **agent_report**: integrator-2 reported control_plane_verified_live_non_durable
- 2026-06-05T22:16:38+00:00 **worklane_status**: worklane#163 -> integration_failed
- 2026-06-05T22:16:38+00:00 **agent_report**: integrator-2 reported integration_failed
- 2026-06-05T22:16:40+00:00 **worklane_created**: Fix failing tests from run 756
- 2026-06-05T22:16:40+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:16:44+00:00 **poke**: Queued message for developer-2
- 2026-06-05T22:16:44+00:00 **poke_delivered**: Delivered poke to 1 agents

## Next steps
Move next worklane forward: Fix failing tests from run 756
