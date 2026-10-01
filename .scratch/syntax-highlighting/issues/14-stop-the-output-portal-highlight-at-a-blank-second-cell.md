# 14 — Stop the Output Portal highlight at a blank second Cell

**What to build:** A Sequence-capable Output Portal's highlight stops at the first blank Cell, as `12` decided and the highlight's own documentation states. Today the extension tests only the first Cell of each following pair and steps by two, so a blank Cell at an odd offset is never inspected: a one-Cell gutter after an odd-length run is stepped over, and the highlight tints the neighbouring Expression.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] A blank Cell at either offset of a following pair ends the run, and the pair it falls in is taken whole, as `12` decided.
- [x] A regression test covers the case that escaped: a 20×2 Grid with `":-0104"` over `"010203040 .+0304"`, a one-Cell gutter at column 9 after an odd run. The highlight covers columns 0–9 of the second row and not 10–15.
- [x] The existing even-offset (`narrow`) and two-Cell-gutter (`straddled`) cases still pass unchanged.
- [x] The public rustdoc on `RenderCell::output_portal` states the shipped rule (the run of written Cells, stopping at the first blank Cell and clipped to the Reservation). The old "each following written Cell pair" phrase is already gone (`orcvs/src/render_frame.rs:75-86` now defers to `SourceRevision::output_portal_highlight` without stating the rule), so what is left is to state it. This line moves here from `theming/15`.
- [x] `cargo nextest run --package orcvs --locked` and `--package console` pass. `cargo test --workspace --doc --locked` passes.

## Comments

**2026-09-24 — opened by the release-membership audit.** Found by reading `SourceRevision::fitted` against `12`'s decision. The Tick reserves by Reservation, not by this highlight, so the defect is paint-only, but it contradicts a resolved release decision. Blocks `v1-release/03`.

**2026-09-29 — audit at `cad296df`.** The defect is still present. `SourceRevision::fitted` (`orcvs/src/source/mod.rs:185-194`) starts at `start + OUTPUT_PORTAL_SEQUENCE_MINIMUM_WIDTH`, steps by `SCALAR_WIDTH` and tests only `self.written(fitted)`, the first Cell of each pair, so a blank at the second Cell is never inspected. The doc on `output_portal_highlight` (`mod.rs:165-172`) states the rule the code breaks: "A **blank Cell**, not a wholly blank pair, is what ends the run … One blank Cell ends the answer". No regression fixture exists (`git log -S'010203040 .+0304'` finds nothing). The rustdoc line is narrowed: the stale phrase has already gone from `RenderCell::output_portal`, and what remains is to state the rule there.

**2026-10-01 — resolved.** `SourceRevision::fitted` now inspects every Cell of each following pair: the run continues while the first Cell is written, a blank first Cell ends it before the pair, and a blank later Cell ends it inside the pair, which is taken whole, as `12` decided. `orcvs/src/source/mod.rs`'s `a_blank_second_cell_of_a_pair_stops_the_highlight` is the regression: against the old step it read row 1 as `################....`; it now reads `##########..........`. The `narrow` and `straddled` cases in `a_blank_gutter_cell_stops_the_highlight_whatever_follows_it` pass unchanged. `RenderCell::output_portal`'s rustdoc now states the rule: at least four Cells, then the run of written Cells, stopping at the first blank Cell and completing the pair the run stops inside, clipped to the Reservation.
