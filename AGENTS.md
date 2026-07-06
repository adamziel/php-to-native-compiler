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
- Do not use broad process dumps while PHPT corpus jobs are running. In
  particular, never run `pgrep -af run-tests.php`, `ps -ef | grep run-tests`,
  `ps -eo ... cmd`, or equivalent commands that can print full PHPT argument
  lists; those lists are large enough to destabilize the interactive Codex
  session. Use
  `tools/phpt-safe-status.sh`, targeted tmux status checks, `status.tsv`, and
  bounded log tails instead.
- Use stable Rust unless a documented reason says otherwise.
