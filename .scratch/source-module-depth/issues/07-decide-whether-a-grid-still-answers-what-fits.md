# 07 — Decide whether a Grid still answers what fits in a row

**What to build:** Either a production caller for `Grid::fits`, or its deletion. Right now it is a
public Grid question with tests and no asker.

**Blocked by:** None.

**Status:** needs-triage

**Tags:** release/v1

- [ ] `Grid::fits` either has a production caller or is gone, and its tests follow it either way.
- [ ] No behaviour changes; this is a question about what a Grid is asked, not about what it
      answers.

## Comments

Issue 02 replaced the partition's `grid.fits(anchor, 2)` guard with a row slice that simply has no
second byte to read at the row's edge — the row edge stopped needing a rule at all. That left
`fits` with its definition, its three tests in `orcvs/src/grid.rs`, and nothing calling it.

It is `pub`, so no lint fires and nothing forces the decision. That is exactly why it needs one
made deliberately: a Grid answering a question nobody asks is the kind of surface that quietly
grows a second, differently-shaped answer later. `spatial-tick-planning` adds producers that
write to Source and may well want to ask whether a result fits its row — `SpanWrite::at` currently
asks `offset_in_row` instead — so the honest options are a caller or a deletion, not a shrug.
