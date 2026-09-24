# 14 — Stop the Output Portal highlight at a blank second Cell

**What to build:** A Sequence-capable Output Portal's highlight stops at the first blank Cell, as `12` decided and the highlight's own documentation states. Today the extension tests only the first Cell of each following pair and steps by two, so a blank Cell at an odd offset is never inspected: a one-Cell gutter after an odd-length run is stepped over, and the highlight tints the neighbouring Expression.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] A blank Cell at either offset of a following pair ends the run, and the pair it falls in is taken whole, as `12` decided.
- [ ] A regression test covers the case that escaped: a 20×2 Grid with `":-0104"` over `"010203040 .+0304"`, a one-Cell gutter at column 9 after an odd run. The highlight covers columns 0–9 of the second row and not 10–15.
- [ ] The existing even-offset (`narrow`) and two-Cell-gutter (`straddled`) cases still pass unchanged.
- [ ] The public rustdoc on `RenderCell::output_portal` states the shipped rule (the run of written Cells, stopping at the first blank Cell and clipped to the Reservation) instead of "each following written Cell pair". This line moves here from `theming/15`.
- [ ] `cargo nextest run --package orcvs --locked` and `--package console` pass. `cargo test --workspace --doc --locked` passes.

## Comments

**2026-09-24 — opened by the release-membership audit.** Found by reading `SourceRevision::fitted` against `12`'s decision. The Tick reserves by Reservation, not by this highlight, so the defect is paint-only, but it contradicts a resolved release decision. Blocks `v1-release/03`.
