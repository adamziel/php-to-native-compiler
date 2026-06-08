# Project Rules

This is the new from-scratch PHP-to-native compiler line.

Read `NEW_PROMPT.md`, `README.md`, and `PROGRESS.md` before changing behavior.

Rules:

- Build generic PHP compiler/runtime semantics; do not shape implementation to
  individual expected test output.
- A task is ready only when it is integrated into the active branch.
- Keep `PROGRESS.md` factual and current after integrated behavior changes.
- Prefer small integrated commits that keep production moving.
- Do not stop all work because one integration problem appears; split work and
  keep independent changes flowing.
- Use stable Rust unless a documented reason says otherwise.

