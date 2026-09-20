# 12 — Fit a Sequence-capable Output Portal to its answer

**What to build:** A Sequence-capable root's Output Portal highlight covers its answer rather than the rest of the row. It shows at least four Cells from the Output Portal, whether empty or written. Past those four, it extends by each following Cell pair that holds written content and stops at the first blank pair. It never reaches past the root's Reservation, so it is clipped at the row edge. A scalar root keeps its Cell pair.

Four is the minimum because a Function that never writes more than two Cells would be declared scalar. Being Sequence-capable only matters when an answer can be longer. It also lets a viewer tell a Sequence-capable root from a scalar one before any Tick.

The Reservation itself is unchanged: Tick scheduling still reserves from the Output Portal to the end of the row (ADR 0036), and `10`'s agreement test still compares the Reservation with the scheduler. Only the highlight narrows, and it still reads the current Source revision alone (`05`).

**Blocked by:** None (can start immediately). `06` and `10` are resolved.

**Status:** resolved

- [x] A Sequence-capable root's highlight covers exactly its written answer where the answer is four Cells or longer: `01020304` south of `:-0104`, `C4c4D4` south of `:#C4D4`, `04030201` south of `:<:-0104`, and `010203` south of `:&.+0001:-0203`.
- [x] An empty Sequence-capable Output Portal shows four tinted Cells before any Tick.
- [x] A one-Atom answer shows four tinted Cells.
- [x] The highlight stops at the first blank Cell pair after the minimum.
- [x] Near the row edge, the highlight is clipped to the Reservation.
- [x] A regression test uses a two-column layout like the one this was found in, where Sequence roots on the left and scalar roots on the right share rows. The left roots' tint does not reach the right roots' Expressions or their scalar Output Portals.
- [x] Scalar roots, the Reservation, Tick scheduling and the agreement test are unchanged. Every existing Tick test passes unchanged.
- [x] `05`'s Width row, the theme documentation and `06`'s Sequence case describe the fitted highlight.

## Answer

`SourceRevision::output_portal_highlight(&self) -> Vec<bool>` (`orcvs/src/source/mod.rs`) answers, in the Grid's row-major order, whether each Cell draws as a root Function's Output Portal. `RenderFrame::derive` reads it in place of `LanguageMap::output_portal_cells`, so `RenderCell::output_portal()` now carries the fitted highlight rather than the Reservation.

The fit is a separate derivation layered on the Reservation, not a narrowing of it. `LanguageMap::output_portal_reservations` is the one Reservation derivation: it answers, per eligible root, its Reservation range and whether that root may answer a Sequence. `LanguageMap::output_portal_cells` flattens the same list for `10`'s agreement test — which is therefore still comparing the geometry the highlight is fitted inside — and is now `#[cfg(test)]`, following `Paint::derive`'s precedent, since no shipped build has a caller for the Reservation as a per-Cell fact. Tick scheduling is untouched.

The fit lives on `SourceRevision` because it needs both inputs at once: the Reservations, which only the Language Map derives, and the Cell contents of this revision, which the Language Map deliberately does not retain.

The rule, for one Reservation: a scalar root's range passes through unchanged — the Sequence-capable flag is carried on the Reservation rather than inferred from the range's length, so a scalar pair is never widened to the minimum and a Sequence-capable root clipped to two Cells or fewer at the row edge is never mistaken for a scalar. A Sequence-capable root takes `min(4, the Reservation)` Cells, then extends by each following Cell pair holding written content and stops at the first blank pair, clipping every step to the Reservation.

**Written** is `SourceRevision::content_at`'s question, so `CellContent::SPACE` is blank: a space reads back identically to a Cell never written and the highlight has no other fact to tell them apart. A pair counts when **either** of its two Cells is written, not only when both are — an answer is delivered as whole Atoms, so a pair with one written Cell holds something a write or an edit left, and stopping mid-pair would draw a written Cell outside the highlight covering the answer it belongs to.

**Tests.** `orcvs/src/source/mod.rs`'s `output_portal_highlight` module pins each rule from Source text with no Tick, reading a row back as `#`/`.`: the four worked examples above, the empty Portal's four Cells, a one-Atom answer's four Cells, the stop at the first blank pair past the minimum, a half-written pair counting, a Reservation the row edge cuts to three Cells, and a scalar root keeping its pair whatever follows it. `console/src/paint.rs`'s `a_left_columns_sequence_tint_never_reaches_the_right_columns_roots` is the two-column regression: a 20x5 Grid with Sequence roots on the left and scalar roots on the right offset one row, so each left root's destination row is a right root's Expression row. Against the whole-Reservation highlight it fails with row 1 reading `####################` where the fit reads `########............`.

`a_sequence_answer_paints_every_cell_and_the_remainder_is_an_empty_output_portal` is now `a_sequence_answer_paints_every_cell_and_stops_where_the_answer_does`: the two Cells the Reservation covers past `04030201` are ordinary untinted blanks. `language_map.rs`'s `a_sequence_capable_root_covers_its_row_to_the_end` and `a_root_widened_by_a_nested_sequence_operand_covers_its_row_to_the_end` are unchanged, because they test `output_portal_cells` — the Reservation, which this ticket does not touch.

## Comments

Found from a screenshot on 2026-09-19. With the highlight covering the whole Reservation, the left column's Sequence roots tinted the rest of every Output Portal row, running under the right column's Tick Expressions and their scalar answers.

Known limit: content written directly after an answer is absorbed into the highlight past the fourth Cell. That covers stale Cells a shorter answer left behind (a Tick writes only the answer's width and does not clear past it) and text written by hand. `13` records the alternative that avoids this.
