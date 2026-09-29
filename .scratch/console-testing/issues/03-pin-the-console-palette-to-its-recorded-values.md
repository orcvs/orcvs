# 03 — Pin the console palette to its recorded values

**What to build:** Assert the twenty-two palette tokens at the exact values `restyle-egui-console/02`
decides and `console/src/theme.md` records, so the record and the code cannot drift apart in silence.

**Blocked by:** 01 — Rename the shell crate to console.

**Status:** resolved

- [ ] Every one of the twenty-two dark tokens is asserted at its exact value in
      `console/src/style.rs`: page, source, grid line, sector line, ordinary, function, bang and
      error, number, note, marker, highlight, the four Cursor bloom fill and line pairs, selection
      fill, the selection stroke while the caret is hidden, and the selection and Cursor stroke.
- [ ] The values asserted are the ones named in `restyle-egui-console/02` and `console/src/theme.md`.
      Where the three disagree, the disagreement is resolved in this change rather than encoded.
- [ ] The test names `console/src/theme.md` as the record it is checking, so a reader knows which
      document a failure implicates.
- [ ] The six existing relationship tests are kept. They state why the palette is shaped as it is;
      this test states what it is. Neither replaces the other.
- [ ] The `marker` token is asserted as a value like the rest, with a comment recording that
      `restyle-egui-console/02` holds no capture to it: nothing produces `Glyph::Marker` on a
      rendering path, so the token is a compile-time requirement for the exhaustive match in
      `cell_visuals` rather than a visual one.
- [ ] A later palette change fails this test, and the same commit must change `console/src/theme.md`
      and `restyle-egui-console/02`.
- [x] The test is written so that a second palette extends it rather than rewrites it.

## Comments

`restyle-egui-console/02` worries in its own comments that its acceptance lines "already pass by
construction" because they were pinned to the shipped `PALETTE` const. This test is what makes that
concern moot in the other direction: after it exists, the shipped const cannot move without the
record moving with it.

This is the cheapest work in the effort and it depends on nothing but the rename. It needs no new
dependency, no harness, and no CI change.

**This issue covers the dark palette only, and two later changes touch it.**
`theming/03` renames `PALETTE` to `DARK_PALETTE`, which this test follows mechanically.
`theming/04` decides a light palette, and its own acceptance line requires extending
this test to pin those twenty-two values too. Neither is a blocker: run first against whichever
constant exists, and let the later issues grow the test. That is what the final acceptance line above
is for.

### Audit at cad296df — 2026-09-29

Resolved, superseded by the theming work. Both built-in Themes are pinned at exact values:
`okabe_ito_defines_every_key_at_the_schema_values` (`console/src/theme.rs:1223`),
`orcvs_light_defines_every_key_at_the_recorded_values` (`:1372`), and the chrome tests
`okabe_ito_chrome_matches_the_decided_record` and `orcvs_light_chrome_matches_the_decided_record`
(`console/src/style.rs:881`, `:1267`). The light suite extends the dark one, which is the one box ticked.

The other boxes are left unticked because their subjects no longer exist, not because the work is
missing: there is no `PALETTE`/`DARK_PALETTE` const, no twenty-two-token set, no `Glyph::Marker`
(the marker and highlight tokens were retired under `retired-glyph-vocabulary`), and `cell_visuals`
is now `cell_visuals_with_cursor_colour`. The dark Source test names `schema.md` as its record rather
than `theme.md`.
