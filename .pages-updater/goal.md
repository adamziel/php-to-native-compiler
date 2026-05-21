# Goal

Keep the GitHub Pages progress report for adamziel/php-to-native-compiler up to date from this sandbox.

The updater must:

- Read `/home/claude/php-to-native-compiler` without modifying it.
- Read lane progress logs under `/home/claude/phpc-lane-*` without modifying them.
- Regenerate the `gh-pages` static report every 15 minutes when source inputs change.
- Commit and push changed Pages output to `origin/gh-pages`.
- Avoid duplicate updater processes.
- Keep operational logs out of Git.
