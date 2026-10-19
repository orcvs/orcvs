# 06: Benchmark the Render Frame with Tick Portals

**What to build:** A benchmark of deriving the Render Frame with a full record
present (reads and writes, current and retained), beside the existing paint and
Source benchmarks, so CI measures the cost the record adds.

**Blocked by:** 03, 05

**Status:** resolved

- [x] A benchmark derives a frame from a Source whose last Tick read and wrote
      across the Grid, current and retained.
- [x] No performance claim is made beyond what the benchmark measures.

## Comments

2026-10-19 — `source_render_frame_tick_portals` in `orcvs/benches/source.rs`,
registered in its `criterion_group!`, derives a Render Frame through
`Orcvs::render_frame` over the dynamic-writer Tick fixture at `TICK_SIZES`,
whose voices each make two selected reads and selected and declared writes per
Tick. Each size is measured three ways over the same Cells:
`none/<cols>x<rows>` (written by hand, no record), `current/<cols>x<rows>` (the
revision the second settling Tick left) and `retained/<cols>x<rows>` (one Cell
rewritten with its own content, so every entry's owner still stands and the
fold asks that of each painted entry). The fixture asserts the Tick is quiet
and that the frame paints four read Cells per voice with a record and none
without, so a fixture that stopped recording fails instead of measuring the
bare derive. CI's bencher parse picks the lines up by name; no bench list or
series file names benchmarks, and `benches/floors.toml` guards none of these,
since no figure has been measured on the hosted runner. No performance claim
is made: the comparison is CI's. Locally the target was compiled with
`cargo bench --no-run --package orcvs --locked`, and the group was run once in
criterion's `--test` mode to check the fixture's assertions, which measures
nothing.
