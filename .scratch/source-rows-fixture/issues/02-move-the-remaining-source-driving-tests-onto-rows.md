# 02 — Move the remaining Source-driving tests onto rows

**What to build:** The `orcvs` tests outside playback, MIDI and the Tick tests that build a Source
only to drive something else state it as rows through `orcvs::source::rows`, as issue 01 did for
playback and MIDI. A test whose subject is a flat index keeps it.

**Status:** needs-triage

**Why:** Issue 01 moved the playback, MIDI and Tick tests. These modules still place Expressions
by flat index into a Grid whose width they state, so an Expression that grows by an operand
changes their widths and indices too:

- `orcvs/src/app.rs` — for example `Grid::with_shape(10, 6)` with `!>007FC4` written at
  `grid.cell_index(20 + offset)`.
- `orcvs/src/render_frame.rs` — about 34 flat-index sites.
- `orcvs/src/source/portal.rs` — about 11 flat-index sites.

- [ ] Each module above is read site by site: a Source that only drives the code under test moves
      to `commander` or `seeded_source` with edits by column and row; a site that tests the index
      itself stays, with the reason clear from the test.
- [ ] Each moved test states the room its Functions write to, and asserts its Tick reports no
      diagnostics where it runs one.

**Out of scope:** `orcvs/src/source/model.rs`, `orcvs/src/source/mod.rs`,
`orcvs/src/source/language_map.rs`, `orcvs/tests/language_map.rs` and `orcvs/src/grid.rs`. They
test the Source model and the Grid, where a flat index is the thing under test.

## Comments

Filed from review of PR #412 on 2026-11-18. The counts are from a grep at `b50d8b0c` and are a
guide to size, not a checklist.
