# 14 — Decide whether a half-typed pair replays the last Note

Status: resolved

**What to decide:**

Typing a Note into a pair Track selects passes through a half-written pair, such as `E ` on the way
to `E5`. Track reads it as partial input, so it is invalid, writes nothing and is diagnosed. Under
ADR 0069 the Timed Play's note slot keeps the Note the previous step wrote, and if the Delay bangs
that Tick the Play sounds that Note again. `05`'s first criterion asked for recovery "without
spurious notes", which this repeat contradicts.

The multi-voice tracker does not repeat: its Read is nested in the Play's note slot, so a partial
read fails the Play and it sends nothing (`orcvs/src/playback/tests/shipped_trackers.rs`,
`a_half_typed_step_silences_only_its_own_voice`).

## Acceptance criteria

- [x] Decide whether a value Function that is invalid because of partial input should leave its
      Output Portal as it was (ADR 0069 today) or clear it, and record the decision in an ADR if it
      changes.
- [x] ~~If it changes, update `console::kittest_tests::live_editing::a_half_typed_note_is_diagnosed_once_and_finishing_it_plays_it`
      and the "While you edit" section of `examples/cell-tracker.md`.~~ It does not change.

## Comments

Split from `05` on 2026-10-17. The behaviour is pinned by the test above and documented in the
guide, so nothing is broken while this waits.

### Decided 2026-10-17 by Toby

Keep ADR 0069: a value Function that is invalid because of partial input writes nothing and leaves
its Output Portal as it was, so a half-typed pair Track selects replays the previous step's Note.
No ADR changes. The test and the guide's "While you edit" already describe this behaviour.
