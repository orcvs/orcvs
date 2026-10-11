# 01 — Carry a Bang through a pass-through as its glyph

Status: resolved

**What to build:**

ADR 0071's first decision. A Copy, Read, Track, Write or Push whose answer is a Bang writes `**` over its complete destination through the ordinary overwrite path, and activates the roots aligned with the destination. The relay branch in `orcvs/src/source/tick/execution.rs` (the `atom == Atom::Bang && (selected || copies_language_unit())` block) is removed; a destination root's anchor is overwritten like any other Cells.

## Acceptance criteria

- [x] A pass-through's `**` onto empty Cells writes `**` and activates the aligned roots, as today.
- [x] A pass-through's `**` onto a root's anchor overwrites the anchor pair, does not activate that root, and activates the roots aligned with the destination. The overwritten root does not run that Tick.
- [x] A pass-through's `**` onto occupied data overwrites it and activates the aligned roots, with no diagnostic.
- [x] A pass-through's `**` leaving the Grid diagnoses as any out-of-Grid write does.
- [x] A chain `.=` into `=v` into `=v` carries the Bang down the column in one Tick.
- [x] The tests pinning the relay are flipped to the new rule:
  - `orcvs/src/source/tick.rs`: `a_relayed_bang_activates_a_root`, `a_relayed_bang_activates_a_root_above_the_copy`, `a_relayed_bang_on_an_occupied_non_root_diagnoses_and_writes_nothing`, and any other `relayed_bang` test whose expectation depends on the relay.
  - `orcvs/src/source/tick/absolute_copy.rs`: `a_bang_copied_onto_a_roots_anchor_activates_it_without_overwriting_it`.
  - `orcvs/src/source/tick/nested_copy/root_parity.rs`: `a_root_copy_relays_a_same_tick_bang_but_reads_a_typed_bang_as_empty` (its first case; the typed case moves with ticket 03).
  - `orcvs/src/source/tick/nested_copy/vertical_and_map.rs`: `a_nested_vertical_copy_relays_a_same_tick_bang_to_an_aligned_root`.
  - `orcvs/src/source/tick/read.rs`: `a_bang_a_read_reads_is_relayed_and_activates_the_root_it_lands_on`, `a_bang_an_absolute_read_reads_is_relayed`.
  - `orcvs/src/source/tick/track.rs`: `a_bang_track_reads_is_relayed_and_activates_the_root_it_lands_on`.
  - `orcvs/src/source/tick/write.rs`: `a_bang_written_onto_a_roots_anchor_activates_it_without_overwriting_it`, `a_bang_written_onto_an_occupied_non_root_diagnoses_and_writes_nothing`.
- [x] CONTEXT.md's Bang and Copy entries state the rule; the Copy entry no longer says a Bang output "activates an Expression root without overwriting it" or "diagnoses at an occupied non-root".
- [x] The dependency schedule orders a root whose anchor a dynamic `**` may cover after the writer, as for any value write onto an anchor; a test pins it for a Write and a Copy.

## Comments

- The criterion "The overwritten root does not run that Tick" holds when the writer goes first. A covered root that feeds the writer has already taken its Turn when the `**` lands, so the write is feedback and that root meets the `**` on the next Tick.
