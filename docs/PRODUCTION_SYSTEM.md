# Production System

The project treats integration as the unit of progress.

Principles:

- Work should flow in small branch-ready units.
- Every station must expose defects quickly.
- A defect should trigger containment and splitting, not a total stop.
- Avoid inventory: patch piles, stale worktrees, and unpushed local changes are
  not progress.
- Keep the main branch releasable enough to push frequently.
- Use `PROGRESS.md` as the public and remote-visible production board.

Operational loop:

1. Choose the next generic semantic capability.
2. Implement the smallest integrated slice.
3. Add tests that prove the semantic behavior.
4. Update `PROGRESS.md`.
5. Commit and push.
6. Start the next slice without waiting for unrelated blockers.

