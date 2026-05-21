# Memory

- The public site is served from `origin/gh-pages` at `https://adamziel.github.io/php-to-native-compiler/`.
- The Pages checkout lives at `/home/claude/php-to-native-compiler-pages`.
- The active compiler checkout lives at `/home/claude/php-to-native-compiler`; treat it as read-only because other sessions may be writing there.
- The report generator is `tools/build-site.mjs`.
- The generated browser refresh interval is 15 minutes (`secs=900`) and is independent from the updater loop.
- The updater loop uses a content fingerprint so it does not push timestamp-only churn when source inputs are unchanged.
