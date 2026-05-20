# Project Rules for Codex Sessions

This repository is a real PHP-to-native compiler effort, not a demo scaffold.

Never expose the local network by starting or using a remote tunneling service
such as ngrok, cloudflared tunnels, localtunnel, serveo, localhost.run,
lhr.life, Tailscale Funnel, or similar tools. Use only the sandbox-provided
`8080` ingress and local-only proxies inside the sandbox.

At the beginning of each session, read:

- `AGENTS.md`
- `docs/PROGRESS.md`
- `docs/ARCHITECTURE.md`
- `docs/SUPPORT.md`
- `README.md`
- `docs/LOOP_MEMORY.md` when running under unattended loop automation

Engineering rules:

- Do not claim support for a feature unless executable code and tests prove it.
- Prefer a small correct subset over broad placeholder claims.
- Keep dynamic PHP features as runtime fallback zones, not excuses to stop.
- Use stable Rust for the compiler and runtime unless a specific document records a stronger reason.
- Use crates only when they materially reduce work.
- Do not replace the parser with regular-expression parsing.
- Unsupported edge cases must be named in docs.
- After meaningful behavior changes, run tests, fix failures, update docs, and update `docs/PROGRESS.md`.

Feature completion requires:

1. Implementation code.
2. Tests.
3. CLI exercise path.
4. Accurate documentation.
5. Named unsupported edge cases.

Current discipline:

- `phpc run` may use the interpreter/runtime path for supported Milestone 1 syntax.
- `phpc compile --emit-ir` and `--emit-asm` are intentionally narrower than `run`; they must reject unsupported lowering instead of emitting misleading native code.
- Runtime values must remain PHP-shaped: boxed values first, ordered int/string keyed arrays later, references and copy-on-write added when implemented.
- Operational automation is documented in `docs/OPERATIONS.md`; checkpoint
  commits must go through `tools/checkpoint.sh`.
