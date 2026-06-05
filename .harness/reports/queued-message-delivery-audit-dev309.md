# Queued Message Delivery Audit for Manager Assignments

Agent: developer-309
Lane: 126
Timestamp: 2026-06-05T09:00Z
Scope: read-only control-plane audit. No compiler/runtime edits, no scheduler
edits, no PHPT/full-suite gate.

## Summary

Targeted manager assignment messages are not a reliable delivery path right now.
For messages `439..508`, all 70 targets have live `agents` rows and tmux window
names, but 70/70 also have `last_prompt_at` earlier than the message timestamp.
As of the audit query, most post-launch messages were still `queued`, while
many corresponding `agents.current_status`, `agents.notes`, and `work_lanes`
rows had already been updated directly by manager-side SQLite writes.

That means active agents are often learning assignments only if they inspect
SQLite shared memory (`work_lanes` / `agents.notes`) rather than because their
targeted message was delivered into the running Codex session.

## Evidence

Data sources:

- SQLite database:
  `/home/claude/php-to-native-compiler/.harness/harness.sqlite3`
- Current launch prompt file:
  `/home/claude/php-to-native-compiler/.harness/prompts/developer-309.md`
- `tmux list-windows -t 1`
- Top-level `.harness` file listing. The only top-level Python harness file
  visible without descending into worktrees was
  `.harness/tests/test_codex_command.py`; scheduler delivery code was not
  inspected for this report.

The current `developer-309` launch prompt contains only:

- assigned work: `Maintain Developer capacity`
- no lane 126 text
- no manager-18 targeted assignment message

So lane 126 was discovered from `agents.notes` / `work_lanes`, not from the
launch prompt.

## Message Status Snapshot

Query window: `messages.id >= 439`.

Final observed aggregate at audit time:

| Scope | Total | Queued | Delivered | Sent | Other statuses |
| --- | ---: | ---: | ---: | ---: | ---: |
| All messages `>=439` | 70 | 42 | 13 | 1 | 14 |
| Assignment batch `457..478` | 22 | 16 | 2 | 1 | 3 |
| Reserve batch `479..508` | 30 | 11 | 10 | 0 | 9 |

The `messages.status` column is not normalized. In the same window I observed
status values beyond scheduler-looking states: `acknowledged`, `read`, `done`,
`completed`, `delivered_applied`, `processed`, and several
`processed_by_*` variants. These values changed while the audit was in
progress as agents updated shared memory. Treat `delivered`/`sent` as weak
evidence unless paired with a prompt/tmux delivery record.

All 70 target rows were live:

```text
messages >=439 total: 70
live agent rows:       70
rows with tmux_window: 70
last_prompt_at < ts:   70
```

The `last_prompt_at` result is the critical signal: every targeted message in
this window was created after the target's recorded prompt time. Either
post-launch delivery does not update `last_prompt_at`, or the messages were not
delivered through the prompt path. In both cases, `last_prompt_at` cannot prove
successful delivery.

## Assignment Batch `457..478`

Manager-18 created/reassigned lanes 110..132 at `2026-06-05T08:55:47Z`.

Findings:

- 22 targeted assignment messages were inserted.
- 16 remained `queued`.
- 3 were `delivered` or `sent`.
- 3 had free-form processed statuses.
- 18 target agents had lane assignment evidence in `agents.notes`.
- 19 target agents had lane assignment evidence in `agents.current_status`.

Important mismatches:

| Message | Target | Lane | Message status | Agent-side result |
| ---: | --- | ---: | --- | --- |
| 460 | developer-297 | 113 | queued | still `reserve_ready_clean_pushed`; later event says no assigned lane |
| 463 | developer-300 | 116 | queued | still `reserve_capacity_available`; later event says no assigned lane |
| 466 | developer-303 | 120 | queued | status says awaiting targeted assignment |
| 467 | developer-304 | 121 | queued | still `reserve_clean_pushed` |
| 472 | developer-309 | 126 | queued | lane discovered from DB notes/work_lanes, not message delivery |

This shows two behaviors in the same batch:

- Manager-side DB writes can make a lane visible without message delivery.
- Some agents can finish a reserve path and report no assigned lane even after
  their `work_lanes` row exists, when the direct message remains queued and
  their agent row was not updated with lane context.

## Earlier Manager-18 Batch `439..456`

Manager-18 reassigned lanes 8, 68, 69, 73, 74, 78, 81, 82, 83, 85, 86, 87,
95, and 106..109 at `2026-06-05T08:54:26Z`.

Findings:

- 18 targeted messages were inserted.
- 17 remained `queued`.
- 1 was `delivered` (`developer-293`).
- The target agents nevertheless mostly show assigned-lane state in
  `agents.current_status`, indicating DB-state visibility independent of the
  message queue.

## Reserve Batches `479..508`

Manager-18 then sent reserve/no-source-edit instructions during the spawn/load
storm.

Findings:

- Message statuses diverged into non-uniform values, including
  `acknowledged`, `read`, `done`, and `processed_by_developer_325`.
- Many reserve agents correctly updated their own `current_status` to
  `reserve_no_source_edits` while their message row still remained `queued`.
- This reinforces that agents are using shared memory inspection or manual
  status updates, not a single reliable message-delivery state machine.

## Conclusion

The current assignment path is split:

1. Manager-side lane assignment writes `work_lanes`, and often also writes
   `agents.current_status` / `agents.notes`.
2. Targeted messages are inserted into `messages`.
3. Running agents do not consistently receive or acknowledge those messages via
   a normalized delivery mechanism.

This makes `messages.status='queued'` ambiguous but operationally important:
the assignment may already be visible through `work_lanes`, or the agent may be
unaware and continue reserve behavior.

## Deterministic Next Actions

Recommended control-plane fixes:

1. Define one normalized message lifecycle, for example:
   `queued -> sent_to_tmux -> acknowledged_by_agent -> completed`.
2. On post-launch tmux delivery, update a separate timestamp such as
   `last_message_sent_at` or update `last_prompt_at` only if that field is
   intended to include injected prompts.
3. Make manager lane assignment writes atomic across:
   `work_lanes`, `agents.current_status`, `agents.notes`, and `messages`.
4. Add a scheduler health check that flags:
   live target, queued targeted message, `last_prompt_at < messages.ts`, and no
   matching lane text in `agents.notes/current_status`.
5. Avoid relying on free-form `messages.status` values for scheduler decisions;
   use an enum or a separate agent-ack event table.

No public PHPT score movement is claimed from this report.
