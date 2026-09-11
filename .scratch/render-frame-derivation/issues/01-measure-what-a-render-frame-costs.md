# 01 — Measure what a Render Frame costs before changing anything

**What to build:** A recorded baseline for `RenderFrame::derive` across the Grid sizes a resizable
console would plausibly offer, through the benchmark group that already exists.

**Blocked by:** None.

**Status:** ready-for-agent

- [ ] `orcvs/benches/source.rs`'s `source_render_frame` group (`:283-295`) covers sizes past the
      current `SIZES = [(16,16), (32,32), (64,64)]` (`:20`), far enough to show whether the cost
      curve is the flat O(Cells) the code reads as.
- [ ] The added sizes are justified in the file, not chosen by feel. They exist to answer what
      derivation costs at a resizable Grid's upper end, and the spec says no bound has been chosen
      yet, so say what the sizes are standing in for.
- [ ] Whether the other benchmark groups need the same sizes is decided explicitly rather than by
      copying. `source_read_revision` (`:269`) is the one that would show the whole-Source clone.
- [ ] The measurement is recorded where a later issue can cite it. Per `CLAUDE.md` and
      `.scratch/benchmarks/spec.md` the comparison lives in CI, so a local run produces a number that
      decides nothing — say where the recorded baseline comes from.
- [ ] Nothing in `orcvs/src/` changes. This issue only measures.

## Comments

Written first so the rest of this effort argues from numbers. The unusual thing here is that the
harness already exists and has since before the question was asked — `source_render_frame` calls
`orcvs.render_frame()` directly, which is the whole of what this effort is about.

The three costs the spec names will not show up equally. The per-Cell walk and the `Vec` per row
scale with the Grid; the whole-Source clone in `read_revision` scales with the Grid too but through a
different mechanism and shows more clearly in `source_read_revision`; `cursor_bloom`'s unconditional
`cell_hash` is a constant per Cell and will hide inside the walk unless something separates it. Do
not try to separate them here — measure the whole first, and let the shape of the curve say whether
separating them is worth an issue.
