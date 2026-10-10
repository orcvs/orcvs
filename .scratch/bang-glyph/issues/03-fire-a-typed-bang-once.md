# 03 — Fire a typed Bang once

Status: resolved

**What to build:**

ADR 0071's third decision. Source remembers which Cells the previous Tick wrote as Bang display. At the start of a Tick, a standalone `**` that is not in that set (typed, or loaded from a Source File) is a Bang at its position: it activates its aligned roots during the Tick and is cleared. Display the previous Tick wrote is cleared before any Turn without activating anything, as today.

## Acceptance criteria

- [x] A typed standalone `**` above a Raw Play plays it on the next Tick and is cleared; on the Tick after, nothing plays.
- [x] A `**` written by Equality in Tick T activates its aligned roots in T only; its display is cleared in T+1 without activating them again.
- [x] A `**` loaded from a Source File fires once on the first Tick.
- [x] Editing a Cell that holds Bang display into `**` again counts as typed and fires once.
- [x] A `**` in a typed operand stays invalid syntax and activates nothing; `a_bang_rejected_in_a_typed_operand_never_activates` (`orcvs/src/source/tick.rs`) stays green.
- [x] The tests pinning the inert manual Bang are flipped: `manually_entered_bang_is_display_only_and_never_activates_midi` and `test_manual_bang_is_inert_at_either_vertical_position` (`orcvs/src/source/model.rs`), `a_stale_bang_does_not_activate_halt` (`orcvs/src/source/tick.rs`) where its `**` was typed, and the typed case of `a_root_copy_relays_a_same_tick_bang_but_reads_a_typed_bang_as_empty` (`orcvs/src/source/tick/nested_copy/root_parity.rs`).
- [x] The remembered set is Source state that survives between Ticks and is reset by a Source replacement or load; persistence stores spellings only, so a reload treats every `**` as typed.
- [x] CONTEXT.md's Bang entry no longer says manually entering `**` is a no-op.
