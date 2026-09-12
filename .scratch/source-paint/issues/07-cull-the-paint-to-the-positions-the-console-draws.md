# 07 — Cull the paint to the Positions the console draws

**What to build:** `Paint::derive`, `Paint::background_runs` and `SourceShapes::new` each walk the whole Grid, so after `06` a Render Frame costs three whole-Grid walks where the loop `06` deleted cost one culled walk. All three take the visible Position range, and `Paint` becomes the paint of the Positions the console draws rather than of every Position the Source holds.

This is not new work. `source-grid-rendering/05` landed it as PR #69 (`cda0632`), against the inline draw loop that `06` deletes. This ticket is that decision re-applied to the layer `06` put in the loop's place, and it exists because `06` was written before `05` landed and silently reverses it.

**Blocked by:** 06

**Status:** resolved

### Why all three walks, and not just the shape step

`05`'s claim is that "the cost of a Render Frame follows the viewport rather than the Source: a Grid the console shows a tenth of costs a tenth of the Cell iteration and a tenth of the Shapes". Culling only `SourceShapes::new` restores the Shape half and none of the Cell-iteration half — `Paint::derive` would still call `cell_visuals` once per Position of the Source and allocate a `Vec<CellPaint>` sized to the Grid, which at the 200x200 that resize exists to offer is forty thousand entries built and thrown away per frame. That is the cost `05` was filed against, so a resolution that leaves it in place unblocks this branch without addressing what it was targeting.

Culling the derive is sound for the reason `05` already recorded: everything the derive reads — `cell_visuals(glyph, cursor_bloom, selected, cursor_visible)` and `CellCharacters::character` — is local to its own Cell, so a Cursor outside the drawn range still blooms correctly on the Cells inside it. `RenderFrame::derive` stays uncalled, as `05` says.

### Index ranges may cross into the value layer; a `Rect` still may not

`05`'s `GridViewport::visible_positions(clip, columns, rows)` answers a `VisiblePositions` — two `Range<usize>`, already clamped to the Grid. The clip `Rect` is consumed in `grid_viewport.rs` and never leaves it.

So the rule the spec states — "`Paint` carries no geometry. No `Rect` appears in it" — survives this ticket intact, and it is the rule that matters: it exists so that an assertion about a colour need not acquire a viewport first. A `Range<usize>` acquires nothing. `BackgroundRun` already carries a `columns: Range<usize>` for exactly this reason, and its doc comment already states it: "Columns rather than a rectangle ... putting a `Rect` here would make every assertion about coalescing acquire a viewport first."

What does change is the module doc's second sentence, "nothing here reads a viewport". Restate it as the rule it was standing in for: nothing here reads a viewport's *geometry*, and a `VisiblePositions` is two index ranges the viewport has already resolved.

### What `Paint` becomes

- [ ] `Paint::derive(frame: &RenderFrame, positions: &VisiblePositions) -> Paint` decides and stores a `CellPaint` for the drawn Positions and no others.
- [ ] `Paint` stores the ranges it covers. `cells` is `positions.count()` long, row-major within the sub-rectangle, so `Grid::index` no longer addresses it — the offset arithmetic is `Paint`'s own and stays inside `Paint`.
- [ ] `Paint::cursor() -> Option<Position>`. The `cursor.expect("a Render Frame selects one of its Cells")` is deleted, and with it the claim that the recovery is total. The Render Frame still selects exactly one Cell; this Paint simply need not cover it. Say that in the doc comment rather than leaving a reader to infer it from the `Option`.
- [ ] `Paint` answers its drawn Positions in row order together with their paint, so the shape step neither mints a `Position` from a pair of indices nor unwraps a `Grid::position` per Cell. The index arithmetic has one home.
- [ ] A Paint over an empty range is legal and carries no Cells: `VisiblePositions::empty()` is what `visible_positions` answers for a console showing none of the Grid, and `05` already reaches that case.
- [ ] Do not keep `at(Position)` or `grid()` alive for tests alone. The repository contract bars shipped surface that only a test reaches, and `-D warnings` will say so anyway. Whichever of them production still uses stays; the rest goes, and the value-layer tests use what production uses.

### The coalescing question is settled, and not by measurement

`05` coalesced over the visible columns and said why: "Sliced rather than filtered, so the run above opens at the first drawn Cell and is flushed at the last. A loop that walked the whole row and skipped the Cells outside the range would carry a run in from off-screen instead."

That reason was about an inline `Option<(Color32, Rect)>` accumulator that `06` deletes. Against `06`'s layer the two candidates — clip each run's column range after coalescing the whole row, or coalesce only over the drawn columns — produce **identical** rectangles, and this is provable rather than probable. `SourceShapes` builds a run's rectangle as `Rect::from_min_max(cell_rect(columns.start, row).min, cell_rect(columns.end - 1, row).max)`, and `cell_rect` is a pure function of one column index. Clipping a range to `[c0, c1)` and coalescing over `[c0, c1)` therefore hand `cell_rect` the same two endpoints, and `background_run`'s device-scale snapping is applied to that rectangle afterwards and sees nothing else. Neither candidate can move a pixel the other does not.

So the choice is cost alone, and coalescing over the drawn columns is the one that is cheaper.

- [ ] `Paint::background_runs()` coalesces over the drawn Positions only.
- [ ] The equality above is asserted as a value fact in `paint.rs`, with no egui: the runs a range-scoped Paint answers are the whole-Grid runs intersected with that range. This is what `3c2b640` would ask for — it exists to pin the one-Cell margin's purpose with an assertion rather than a comment — expressed at the layer that can now hold it.

### The shape step

- [ ] `SourceShapes::new` iterates the drawn Positions. `Vec::with_capacity` takes the drawn count, not `grid.count()`.
- [ ] The Cursor's border is routed to the `cursor` group when the Cursor is among the drawn Positions and nowhere otherwise, which falls out of `cursor()` being an `Option`.
- [ ] The `debug_assert!` on a run covering at least one column stays. It is about `background_runs`' fold, which this ticket changes.
- [ ] Nothing about what appears on screen changes at any zoom or pan: same colours, same geometry, same order, same clip.

### The three tests `05` left behind

`06` deleted the shape-filtering helpers `stroked_cell_rects`, `background_runs` and `background_at`, and three tests `05` added still call them, so the branch does not compile until they are re-expressed:

- [ ] `the_draw_loop_paints_the_visible_range_rather_than_the_whole_source` — counts Positions reached and Shapes emitted at the fit and at `MAX_ZOOM`. Most of it is now a value-layer question: `Paint` can be counted directly, with no `Context` and no shape list.
- [ ] `a_zoomed_row_fills_every_cell_the_viewport_shows_and_no_other` — the run-leak test. Its subject is `background_runs`' fold over a range, which is now assertable in `paint.rs`.
- [ ] `a_zoomed_console_paints_every_sector_seam_inside_the_clip` — keep whatever part of this genuinely needs a console pass and move the rest down.
- [ ] Add what `05` could not: a test that counts the `CellPaint`s a culled derive builds. `05` asserted its claim by counting Positions and Shapes through a console pass because that was the only surface it had; `06` makes the Cell-iteration half countable with neither an `Orcvs` nor a `Context`.
- [ ] `06`'s bar holds — no test asserting what colour a Cell is builds a live `Orcvs` or an egui `Context` — and this ticket must not be the one that quietly buys a test back by building one.

### Out of scope

- [ ] `RenderFrame` is not culled and gains no `selected()` accessor. Making `cursor()` an `Option` is what removes the need for one, and the whole-Grid derive of the Render Frame itself is `render-frame-derivation`'s subject, not this one.
- [ ] `GridViewport::visible_positions` and `VisiblePositions` are unchanged. They landed with `05` and compile standalone.
- [ ] The benchmark this change's cost claim owes is `09`.
- [ ] The ADR and glossary text is `08`.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package console --locked
```

`orcvs` is untouched, so it owes neither a run of its own nor `cargo test --workspace --doc --locked`.

Deferred to CI and named on the `Not run` line: `mise run check_wasm` — nothing here is platform-conditional, there is no `cfg` and no new dependency. The persistence arm is not owed; this touches no feature.
