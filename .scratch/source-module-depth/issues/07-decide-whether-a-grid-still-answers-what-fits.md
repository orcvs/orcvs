# 07 — Decide whether a Grid still answers what fits in a row

**What to build:** Delete `Grid::fits` and the tests that are its only callers. The decision is
made: a Grid stops answering what fits in a row, because production code asks `offset_in_row`
instead.

**Blocked by:** None.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] `Grid::fits` is gone from `orcvs/src/grid.rs`, and `test_grid_answers_whether_a_width_fits_in_the_row`
      goes with it.
- [ ] No production call site changes, because there is none: `SpanWrite::at` and
      `LanguageMap::derive` already ask `offset_in_row`.
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

### Decision: delete it

Taken during the `release/v1` issue alignment on 2026-09-04. `Grid::fits` has no production
caller. Its only callers are the six assertions inside its own test,
`test_grid_answers_whether_a_width_fits_in_the_row` at `orcvs/src/grid.rs:618-627`. The question
it answers is already asked elsewhere, in the form production code needs: `SpanWrite::at`
(`orcvs/src/source/tick.rs:194`) uses `offset_in_row` to reject a write that would cross the row
edge, and `LanguageMap::derive` (`orcvs/src/source/language_map.rs:380`) uses it to bound a Span.
`fits` is a second, differently-shaped answer to that one question, which is the surface this
effort exists to remove.

The two open options in the original statement are therefore settled as the deletion, and the
issue is `ready-for-agent` on that basis. The `release/v1` tag stays: `v1-release/03` names this
issue as a blocker, and `scripts/roadmap.ts` throws when a tagged open issue names a blocker that
is not itself open and tagged.

The paired change lives in `property-testing/02`: its sixth acceptance line named `fits(p, width)`
and now names `offset_in_row`. That ordering is encoded rather than described — `property-testing/02`
lists this issue as a blocker — so the deletion lands before the property suite starts and the two
issues cannot specify opposite things.
