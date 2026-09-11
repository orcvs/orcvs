# 01 — Measure what a Render Frame costs before changing anything

**What to build:** A recorded baseline for `RenderFrame::derive` across the Grid sizes a resizable
console would plausibly offer, through the benchmark group that already exists.

**Blocked by:** None.

**Status:** resolved

- [x] `orcvs/benches/source.rs`'s `source_render_frame` group (`:283-295`) covers sizes past the
      current `SIZES = [(16,16), (32,32), (64,64)]` (`:20`), far enough to show whether the cost
      curve is the flat O(Cells) the code reads as.
- [x] The added sizes are justified in the file, not chosen by feel. They exist to answer what
      derivation costs at a resizable Grid's upper end, and the spec says no bound has been chosen
      yet, so say what the sizes are standing in for.
- [x] Whether the other benchmark groups need the same sizes is decided explicitly rather than by
      copying. `source_read_revision` (`:269`) is the one that would show the whole-Source clone.
- [x] The measurement is recorded where a later issue can cite it. Per `CLAUDE.md` and
      `.scratch/benchmarks/spec.md` the comparison lives in CI, so a local run produces a number that
      decides nothing — say where the recorded baseline comes from.
- [x] Nothing in `orcvs/src/` changes. This issue only measures.

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

---

Resolved. `orcvs/benches/source.rs` gained `FRAME_SIZES`, which is `SIZES` plus 128x128 and
256x256, and `source_render_frame` and `source_read_revision` now run over it. Nothing in
`orcvs/src/` changed.

**Why those two shapes.** The file carries the argument; the short form is that no resize bound
exists to size the series against — the cap belongs to a resize spec that has not been written, and
resize itself waits on `.scratch/grid-boundedness/`. So the shapes stand in for an undecided upper
end rather than claiming one: 256x256 is 65,536 Cells, sixty-five times what a console opens on and
past the 200x200 this effort's spec names as where the question starts to matter. Each step still
quadruples the Cell count, so the five points span a 256-fold range, which is what separates flat
O(Cells) from anything steeper.

**What the other groups got, and why.** `source_read_revision` moved to `FRAME_SIZES` with them.
It is the only other group that earned the larger shapes: `Orcvs::render_frame` reads a revision
and then derives over it, so the whole-Source clone is already inside every `source_render_frame`
number, and measuring it at the same shapes is the only way to subtract it back out without a
benchmark group that pulls `derive` apart — which this issue rules out. `source_edit_rebuild_valid`
and `source_edit_rebuild_invalid` stayed on `SIZES`; the Tick series kept `TICK_SIZES`. Neither was
left alone because it is cheap. The edit path turns out to follow the Cell count too — see the
finding below — but that is a question about editing, not about a Render Frame, and each shape added
there needs a second fixture of that shape in each of two groups. The Tick series iterates
Expressions rather than Positions, so a larger Grid is not what stresses it.

**Where the baseline comes from.** Not from this branch. The recorded baseline is the series the
`Benchmark` workflow stores on `gh-pages` under the `lang` key and charts at
`orcvs.github.io/orcvs/dev/bench`. The four new points — `source_render_frame/128x128` and
`/256x256`, `source_read_revision/128x128` and `/256x256` — begin storing the first time this lands
on `main`; until then a pull-request run finds nothing to compare against and the action skips the
comparison rather than failing, so a green benchmark job on the pull request that merges this proves
nothing about them. The three existing shapes keep their names and therefore their stored history.

**The numbers below are a local observation, not a baseline.** One run, on one Apple Silicon
machine, through the bench binary with the same flags `mise run bench` passes (`--warm-up-time 0.5
--measurement-time 1 --sample-size 10 --nresamples 1000`). `CLAUDE.md` and
`.scratch/benchmarks/spec.md` both say the comparison lives in the action, so these decide nothing
and no later issue should cite them as the thing it beat. They are recorded because the question
this issue was opened to answer is the *shape* of the curve, and one run answers that.

```text
source_render_frame/16x16          1,676 ns/iter (+/-    25)
source_render_frame/32x32          5,856 ns/iter (+/-   474)
source_render_frame/64x64         24,516 ns/iter (+/- 3,348)
source_render_frame/128x128      101,017 ns/iter (+/- 6,957)
source_render_frame/256x256      364,256 ns/iter (+/- 3,819)

source_read_revision/16x16            24 ns/iter (+/-     0)
source_read_revision/32x32            43 ns/iter (+/-     1)
source_read_revision/64x64            77 ns/iter (+/-     3)
source_read_revision/128x128         332 ns/iter (+/-     7)
source_read_revision/256x256       1,190 ns/iter (+/-    10)
```

**The curve is flat.** Per Cell, a Render Frame costs 6.5, 5.7, 6.0, 6.2 and 5.6 ns across the five
shapes — a 256-fold change in Cell count moving the per-Cell cost by under 20%, with no trend. Every
step multiplies the total by roughly four, which is what the Cell count does. `derive` is the flat
O(Cells) the code reads as, and nothing in it is quadratic or hiding a whole-map scan per Cell. In
absolute terms 65,536 Cells derive in 364 µs, which is 2% of a 60Hz frame, so nothing here is a
deadline problem at any Grid this measurement reached.

**The whole-Source clone is not where the time goes.** `read_revision` is 1,190 ns of a 364,256 ns
frame at 256x256 — 0.33%, and its share *falls* as the Grid grows (1.4% at 16x16, 0.31% at 64x64).
It is a `memcpy` of one byte per Cell against a per-Cell walk that builds a struct per Cell; the
walk wins by three orders of magnitude. Whatever else the effort does, an issue aimed at the clone
would be chasing a third of a per cent.

**What that means for `02-stop-paying-for-the-bloom-at-every-cell.md`.** Its `needs-triage` was
hedged on the bloom being invisible next to the other two costs. Half of that hedge is now gone: one
of the two, the clone, is not a cost at all, so everything a frame spends is the per-Cell walk and
the `Vec` per row — which is exactly where `cursor_bloom`'s unconditional `cell_hash` sits. It is
the only candidate inside `derive` this measurement leaves standing, and on that ground it is worth
promoting past `needs-triage`. Two warnings for whoever does. First, the prize is a fraction of
about 6 ns per Cell, and the issue's own third acceptance line — that it may close because the
difference is too small to measure — stays a live outcome. Second, measure it at 256x256 and not at
64x64: the largest shape has the tightest spread of the group here (±1%) while 64x64 ran at ±14%,
which would swallow the whole effect. The CI comparison is looser still, alerting at 150%, so a
change of this size will never be visible there — it has to be argued from a local before-and-after
on one machine, stated as such.

**A finding outside this effort, recorded because it was measured.** The cost of building a
benchmark fixture is what exposed it. Populating a Source is one accepted Cell edit at a time, and
`LanguageMap::rebuild` (`orcvs/src/source/language_map.rs:201-211`) walks every row and re-stamps
the clean ones with the new revision id rather than skipping them, so one keystroke is O(Cells) and
populating a Grid is O(Cells²). Timed locally across the whole bench binary with `--bench --list`,
which builds every fixture and measures nothing, constructing them took 1.5s at the editing shapes,
4.7s with 128x128 added, and 62.9s with 256x256 added. The comment in `Source::edit` — "One Cell
changes one row, and a row is the largest thing a Cell can change" — is true of the Source bytes and
not of the Language Map rebuilt from them. Whether a keystroke should cost the whole Grid is a
question for the edit path, and it is not this effort's.

**The CI cost of this change, stated rather than hidden.** From the numbers above, 256x256 alone
adds about 58 seconds of fixture construction per pass, and a benchmark job makes two passes — the
`bench_warmup` discard run and the measured one — so roughly two minutes per run of
`.github/workflows/bench.yml`, on every push to `main` and every pull request touching `lang/**`,
`orcvs/**`, the manifests, the toolchain, or `mise.toml`. 128x128 adds about six seconds on the same
accounting. The measurement itself is minor beside that: four more benchmarks at the configured
budget is about ten seconds across both passes. The job's timeout is 30 minutes and it is nowhere
near it, so this does not blow the budget, but it is real time on a tier that
`.scratch/verification-gaps/` already has open questions about. The fallback is named in the bench
file: dropping 256x256 recovers nearly all of it and still leaves four points showing the same flat
curve, at the price of never measuring the size the spec's question is actually about.
