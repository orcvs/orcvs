# Make Render Frame derivation cost something other than the whole Source

**Goal:** Settle what `RenderFrame::derive` costs per Render Frame, and whether that cost can follow
something smaller than the total Cell count, before a resizable Grid makes the answer matter.

## Why

`RenderFrame::derive` (`orcvs/src/render_frame.rs:74-112`) builds a `Vec<Vec<RenderCell>>` by
iterating every Position of the Grid, once per Render Frame, with nothing cached between frames.
`Orcvs::render_frame` (`orcvs/src/app.rs:217-227`) constructs a fresh `RenderFrame` on every call,
and the console calls it once per pass (`console/src/console.rs:1018`).

Three things inside that walk looked worth naming separately. Issue 01 measured them, and the answer
is narrower than this spec first claimed — read that correction before planning against the list.

`SourceCommander::read_revision` (`orcvs/src/source/mod.rs:159-166`) clones the whole Source string
each call (`orcvs/src/source/model.rs:266-268`). The Language Map beside it is `Arc`-shared; the text
is not. **This turned out not to be a cost.** Measured beside the frame it feeds, the clone is around
0.33% of a Render Frame at 256 by 256, and its share *falls* as the Grid grows. It is named here
because the code reads alarming, and struck from the list because the measurement says so. Do not
open an issue against it.

`cursor_bloom` (`orcvs/src/render_frame.rs:119-134`) pays its Chebyshev distance, `signal_breakup`
and `cell_hash` for **every** Cell, not only those inside the bloom radius.
`classify_cursor_bloom` (`:174-189`) returns `None` for distant Cells, but only after the arithmetic
is done. The radius is Chebyshev over `DEFAULT_HIGHLIGHT_DOT_SPACING = 7` (`orcvs/src/opts.rs:6`),
so it reaches 15 by 15 — at most 225 Cells of the default 1000 can answer anything but `None`.

The allocation shape is one `Vec` per row plus one outer `Vec`, per Render Frame
(`orcvs/src/render_frame.rs:108-110`).

**What issue 01 found.** Derivation is flat O(Cells) — around 6 ns per Cell with no trend across a
256-fold change in Cell count, and nothing quadratic. So the whole of what a frame spends is the
per-Cell walk and the `Vec` per row, which leaves `cursor_bloom`'s unconditional `cell_hash` as the
only candidate inside `derive` the measurement does not dismiss. That is why issue 02 exists and why
the clone does not have an issue of its own. The prize is a fraction of ~6 ns per Cell, so "too small
to measure, and closed" remains a live outcome for issue 02 rather than a formality.

## Why now, and why not sooner

Nothing here hurts at 40 by 25, and `.scratch/source-grid-rendering/` deliberately claims structure
rather than speed for exactly that reason. What changes the question is a resizable Grid: at 200 by
200 this is forty thousand Cells per frame, and at any bound a resize feature would plausibly offer
it is more. Culling the console draw loop
(`.scratch/source-grid-rendering/issues/05-cull-to-the-visible-position-range.md`) does not reach
any of it — that issue culls the draw loop and explicitly not derivation, because everything the
draw loop reads is already local to its Cell.

Resize itself is gated on the `GridId` rearchitecture that
`.scratch/grid-boundedness/issues/01-decide-whether-the-grid-edge-is-a-language-concept.md` decides.
This effort is not. Measuring what derivation costs is free of that decision and is worth having
before it is made.

## Rules

**A number first.** `CLAUDE.md` requires a benchmark or profile for any claim about cost, and
`.scratch/benchmarks/spec.md` puts the comparison in CI. Unusually for a performance effort, the
harness already exists: `orcvs/benches/source.rs:283-295` has a `source_render_frame` group calling
`orcvs.render_frame()` over `SIZES = [(16,16), (32,32), (64,64)]`. Nothing here needs a new harness,
and no issue in this effort may assert a cost it has not measured through that group.

**`lang` is not involved.** Tick planning is already O(expressions) rather than O(Cells), iterating
`map.expressions()` and not Positions — `.scratch/grid-boundedness/spec.md` records the audit. This
effort is about the Render Frame alone.

**No behaviour change.** A Render Frame derived more cheaply must be the same Render Frame. The
console asserts against it directly, and every existing rendering test is the regression suite.

## Deliberately not in scope

**Not culling derivation to the viewport.** `orcvs` has no notion of what the console is showing,
and giving it one would put a presentation concern inside the crate `ADR 0022` keeps free of the
toolkit. If derivation is ever to be viewport-shaped, that is an interface question deserving its
own decision, not a parameter added to `derive`.

**Not a frame cache keyed on geometry.** `.scratch/source-grid-rendering/spec.md` already records
why: the Cursor blink alone makes a geometry-keyed cache replay the wrong phase, and three of the
four shipped bugs a surveyed implementation carried from one are state this console has.
