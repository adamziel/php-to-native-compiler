# Harness Status

Last generated: 2026-06-05T22:10:57+00:00

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
| 631 | Fix failing tests from run 691 | Developer | queued |  | 0.0 |
| 630 | Fix failing tests from run 690 | Developer | queued |  | 0.0 |
| 629 | Fix failing tests from run 689 | Developer | queued |  | 0.0 |
| 628 | Fix failing tests from run 688 | Developer | queued |  | 0.0 |
| 627 | Fix failing tests from run 687 | Developer | queued |  | 0.0 |
| 626 | Fix failing tests from run 686 | Developer | queued |  | 0.0 |
| 625 | Fix failing tests from run 685 | Developer | queued |  | 0.0 |
| 624 | Fix failing tests from run 684 | Developer | queued |  | 0.0 |
| 623 | Fix failing tests from run 683 | Developer | queued |  | 0.0 |
| 622 | Fix failing tests from run 682 | Developer | queued |  | 0.0 |

## Tests
failed — tools/run-tests.sh (error=0, failed=1, passed=0, skipped=0)

## Resource samples
Latest: CPU 7.62%, RAM 13.24%, disk free 8.66 GB.

## Recent events
- 2026-06-05T22:10:38+00:00 **poke**: Queued message for developer-2
- 2026-06-05T22:10:38+00:00 **poke_delivered**: Delivered poke to 1 agents
- 2026-06-05T22:10:39+00:00 **status_publish_failed**: Committed status files but failed to push: To https://github.com/adamziel/php-to-native-compiler.git
 ! [rejected]          HEAD -> master (non-fast-forward)
error: failed to push some refs to 'https://github.com/adamziel/php-to-native-compiler.git'
hint: Updates were rejected because the tip of your current branch is behind
hint: its remote counterpart. If you want to integrate the remote changes,
hint: use 'git pull' before pushing again.
hint: See the 'Note about fast-forwards' in 'git push --help' for details.
- 2026-06-05T22:10:42+00:00 **worklane_created**: Fix failing tests from run 689
- 2026-06-05T22:10:42+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:10:47+00:00 **worklane_created**: Fix failing tests from run 690
- 2026-06-05T22:10:47+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:10:49+00:00 **worklane_status**: worklane#170 -> integration_failed
- 2026-06-05T22:10:53+00:00 **worklane_created**: Fix failing tests from run 691
- 2026-06-05T22:10:53+00:00 **tests_failed**: Full test suite failed; Coordinator should prioritize stabilization lanes
- 2026-06-05T22:10:57+00:00 **poke**: Queued message for developer-1
- 2026-06-05T22:10:57+00:00 **poke_delivered**: Delivered poke to 1 agents

## Next steps
Move next worklane forward: Fix failing tests from run 691
