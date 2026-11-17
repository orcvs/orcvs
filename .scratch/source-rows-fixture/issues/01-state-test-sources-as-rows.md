# 01 — State test Sources as rows

**What to build:** One test-only module that builds a Source from the rows a reader would see and
addresses a later edit by column and row, used by every `orcvs` test that needs a Source only to
drive something else. The Grid's width and each flat index follow from the rows.

**Status:** resolved

**Why:** The playback and MIDI tests wrote strings at flat Cell indices into a Grid whose width
they stated. Adding the legato operand to Monophonic Play (`8ca257dd`, then `8550fa37`) changed
widths and indices in `midi.rs`, `source/model.rs`, `source/language_map.rs` and `playback.rs`,
none of which test legato.

- [x] `orcvs::source::rows` holds `commander`, `write_at`, `at`, `row`, `grid_of`, and the
      `snapshot`, `seeded_source`, `rows_of` and `cell` helpers that `source/tick/fixture.rs` kept
      to itself.
- [x] `playback.rs`, `playback/tests/*` and `midi.rs` build their Sources through it.
- [x] The duplicate `cell(grid, idx)` helpers in `orcvs/src`, `orcvs/tests/allocation.rs` and
      `orcvs/benches/source.rs` are gone.
- [x] The module is gated on `test-grid-shapes`, with the reason recorded in `orcvs/Cargo.toml`.

## Comments

Decided with Toby on 2026-11-17:

- Edits are addressed by column and row, not by flat index and not by restating the Grid.
- `source/model.rs` and `source/language_map.rs` keep their own writers. They test the Source
  model, where a flat index is often the thing under test.
- The module reuses `test-grid-shapes` and adds no third test-only feature. Rows are how a smaller
  Grid gets filled, so the two travel together.

A derived Grid is exactly as large as its rows. Six tests had relied on spare Cells in a larger
stated Grid: a Function writing below itself, or Monophonic Play's blank legato operand. Each now
states that room as an empty row or trailing spaces.
