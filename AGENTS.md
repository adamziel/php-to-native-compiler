# Project Rules

This is the new from-scratch PHP-to-native compiler line.

Read `NEW_PROMPT.md`, `README.md`, and `STATUS.md` before changing behavior.

Rules:

- Build generic PHP compiler/runtime semantics; do not shape implementation to
  individual expected test output.
- A task is ready only when it is integrated into the active branch.
- Do not update `STATUS.md` or `STATUS.html` as routine implementation-lane
  evidence; the dedicated status-dashboard polecat owns those generated files.
- Prefer small integrated commits that keep production moving.
- Do not stop all work because one integration problem appears; split work and
  keep independent changes flowing.
- Run broad or long PHPT/cargo verification only through
  `tools/run-detached-check.sh`; poll `status.tsv` and small bounded log tails
  instead of streaming large outputs into the interactive session.
- Treat interactive output volume as a session stability risk. Before running
  any diagnostic command, make sure the command itself bounds output at the
  source to roughly a screen or two. Prefer exact files and paths over discovery
  commands. Avoid commands that can print hundreds of paths, argv values,
  failures, diffs, or log lines.
- For exploratory text search, prefer `tools/safe-rg.sh PATTERN [PATH...]`.
  It rejects broad workspace roots and truncates output by default. If raw `rg`
  is necessary, use exact files or narrow directories plus a source-side limit
  such as `-m 20`, `--max-filesize`, or a small `head` in the pipeline.
- Do not use broad process dumps while PHPT corpus jobs are running. In
  particular, never run `pgrep -af run-tests.php`, `ps -ef | grep run-tests`,
  `ps -eo ... cmd`, or equivalent commands that can print full PHPT argument
  lists; those lists are large enough to destabilize the interactive Codex
  session. Use
  `tools/phpt-safe-status.sh`, targeted tmux status checks, `status.tsv`, and
  bounded log tails instead.
- Do not run recursive searches from `/home/claude` or other broad workspace
  roots. Broad `rg`/`find` commands can traverse active lanes, logs, vendored
  caches, generated artifacts, and cloned corpora, producing multi-megabyte
  output that destabilizes the interactive session. Search from the specific
  repo/lane first, and only expand by naming a small set of directories with
  source-side output limits.
- Do not list manifest-heavy directories interactively. In this repo, `tools/`,
  `.runtime/`, detached-check roots, PHPT corpora, lane roots, and dashboard
  state directories can contain hundreds or thousands of entries. Use targeted
  paths, `find ... -maxdepth ... | head`, or purpose-built summary scripts
  instead of bare `ls`, recursive `find`, recursive `du`, or broad globs.
- Do not enumerate full PHPT dashboard shard trees in the interactive session.
  In particular, avoid `find .../ptn-full-phpt-dashboard-loop/runs/.../shards`,
  recursive `ls`, broad `rg` over shard logs, or `cat`/`tail` glob output from
  those directories. Use `tools/phpt-safe-status.sh` and
  `tools/phpt-active-failure-summary.py`; inspect an individual shard log only
  when you already know its exact path, with bounded output such as
  `tail -n 120` or `rg -m 40`.
- Do not capture whole tmux panes or large logs. Use `tmux capture-pane` only
  with explicit small ranges, and use `tail -n 120` or less unless there is a
  concrete reason to inspect more. Never `cat` run logs, corpus logs, generated
  reports, or PHPT outputs.
- Do not print full diffs, full test reports, or full generated files into the
  session. Use `git diff --stat`, `git diff --check`, focused path diffs, or
  bounded snippets. If a full artifact is needed, write it to a file and report
  the path plus a short summary.
- Use stable Rust unless a documented reason says otherwise.
