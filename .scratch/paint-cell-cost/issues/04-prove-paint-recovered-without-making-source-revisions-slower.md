# 04 — Prove Paint recovered without making Source revisions slower

**What to build:** Prove on the authoritative benchmark runner that `03` removes the dominant Paint
cost without transferring it to Source revision reads, Source edits, Tick execution, or Render
Frame derivation. Record the result and reconcile the decisions and deferred cache work against the
measured implementation.

**Blocked by:** 03 — Remove the Paint claim cache without caching Paint on the Language Map.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The pull-request Benchmark workflow compares the candidate with the rebased `origin/main` on
      the same runner and records every `paint_derive`, `paint_background_runs`,
      `source_read_revision`, `source_edit_rebuild_valid`, `source_edit_rebuild_invalid`,
      `source_execute_tick`, and `source_render_frame` result.
- [ ] Both fitted and culled `paint_derive` series materially improve in the direction the profile
      in `01` predicts: the pointer-keyed cache and its growth cost are absent from the measured
      path.
- [ ] No Source revision read, valid or invalid edit rebuild, Tick execution, Render Frame, or
      background-run series regresses beyond ordinary runner noise. Any apparent regression is
      rerun and resolved rather than accepted because the Paint series improved.
- [ ] ADR 0050 is amended or superseded to record the measured seam: the Render Frame carries the
      once-derived written answer, while the Language Map remains authoritative for the parser's
      claim and does not retain whole-Grid Paint views.
- [ ] `paint-cell-cost/02` records that its whole-Grid Language Map implementation was rejected by
      the Source benchmark regressions, without erasing the implementation and review history.
- [ ] The remaining claim-cache criterion in `syntax-highlighting/07` is checked only after the
      benchmark comparison proves the cache is gone without transferring unacceptable cost.
- [ ] The final record names the candidate SHA, baseline SHA, workflow run, exact measurements, and
      explicit accept or reject decision.

## Rejected: `ba99abc`, the first `03` implementation

**Decision: rejected.** The workflow went green only because its alert threshold is loose.
Ticket 04 still fails. The Benchmark workflow alerts at 150% and fails at 300%
(`.github/workflows/bench.yml`), so a green run only says that nothing got 1.5x slower. This ticket
rejects cost moved into Render Frame derivation, and `ba99abc` moved it there.

- Baseline: `3650c7d8c463cf1e48fa7d37c8e84b408ad234be`. This is `origin/main`, with the point stored on
  `gh-pages` by `main`'s run [35503614546](https://github.com/orcvs/orcvs/actions/runs/35503614546).
- Candidate: `ba99abc24a07dda29442d11f76e46070304b6f4e`.
- Workflow: [35505440797](https://github.com/orcvs/orcvs/actions/runs/35505440797), job
  `pull-request`. Both attempt 1 and attempt 2 concluded `success`.

Values are criterion's bencher `ns/iter (+/- spread)`. Speedup is `baseline / candidate`.

| benchmark | baseline | attempt 1 | speedup | attempt 2 | speedup |
|---|---:|---:|---:|---:|---:|
| `source_render_frame/16x16` | 14,410 (154) | 15,823 (297) | 0.911 | 15,871 (297) | 0.908 |
| `source_render_frame/32x32` | 56,893 (4,310) | 60,991 (962) | 0.933 | 60,933 (1,083) | 0.934 |
| `source_render_frame/64x64` | 239,643 (2,543) | 250,959 (4,069) | 0.955 | 251,464 (4,370) | 0.953 |
| `source_render_frame/128x128` | 922,920 (9,006) | 1,021,284 (16,798) | 0.904 | 1,037,105 (18,023) | 0.890 |
| `source_render_frame/256x256` | 3,782,733 (141,749) | 4,174,949 (68,166) | 0.906 | 4,538,320 (183,688) | 0.834 |
| `paint_derive/fitted/16x16` | 11,794 (17) | 3,045 (5) | 3.873 | 2,992 (13) | 3.942 |
| `paint_derive/fitted/256x256` | 3,205,954 (6,152) | 1,095,619 (2,030) | 2.926 | 1,079,002 (43,462) | 2.971 |
| `paint_derive/culled/16x16` | 11,702 (23) | 3,047 (6) | 3.840 | 3,027 (30) | 3.866 |
| `paint_derive/culled/256x256` | 11,912 (44) | 3,085 (10) | 3.861 | 3,089 (13) | 3.856 |

`source_render_frame` was slower at every size in both attempts, and the worst point was the largest
Grid. The benchmarks whose code `ba99abc` did not touch make the result worse, not better: across
the 18 `source_edit_rebuild_*` and `source_execute_tick*` points, the pull-request runner was
**1.048x** (attempt 1) and **1.031x** (attempt 2) *faster* than the baseline runner (geometric
means). Normalised by that control, `source_render_frame` is 0.86–0.91x in attempt 1 and 0.81–0.92x
in attempt 2. Paint recovered 2.9–3.9x, and this ticket does not accept that gain as a trade for
the Render Frame cost.

**Runner variance.** Comparing a pull request with one stored point is weak evidence either way.
`main`'s own series moves about 2x on code that did not change: `208c50a` → `710ec95` touched none
of the measured Source or Paint code, yet `source_render_frame/256x256` went from 1,380,142 to
2,750,067 ns and `source_read_revision/128x128` from 149 to 296 ns. The same thing shows inside this
run. `Paint::background_runs` and `SourceRevision` reads are the same code in `3650c7d` and
`ba99abc`, but `paint_background_runs/fitted/128x128` read 0.865 in both attempts and
`source_read_revision/128x128` read 0.847 and 0.833. The two attempts agree with each other and
differ from the baseline, which is the pattern a runner-class difference produces. This is why a
local paired ablation was needed to find the mechanism, and why the untouched benchmarks are
reported beside every comparison below.

## Evidence ledger

Kinds: **measured** (a number from a named run), **derived** (arithmetic on measured numbers),
**hypothesised**, **unvalidated**.

Measurement boundary: `source_render_frame` times `Orcvs::render_frame()`. That is one
`read_revision()` (a Source `String` clone and a Language Map `Arc` clone) followed by
`RenderFrame::derive` over every Position (`orcvs/benches/source.rs`). `paint_derive` times
`Paint::derive_with_colours` over an already-derived frame.

Local runs are diagnostic only, not acceptance evidence. They ran on an Apple M2 under rustc 1.98.1
with the release bench profile, on a machine carrying unrelated load. A throwaway harness built the
`source.rs` fixture text in one `Source::write_cells` and timed `render_frame()` in 60 batches of
about 20 ms. Five rounds of five variants ran interleaved, with the order alternating each round.
Reported: the minimum of each round's minimum batch, and the median of the five medians.

| # | statement | kind | prediction | discriminating experiment | disconfirming result | result |
|---|---|---|---|---|---|---|
| O1 | `ba99abc` slows `source_render_frame` at every size on CI | measured | — | two CI attempts | a size at or above baseline | Held: 0.83–0.96 raw, 0.81–0.92 control-normalised, both attempts |
| O2 | the regression reproduces locally | measured | B slower than A at every size | A=`3650c7d` vs B=`ba99abc`, interleaved | B within A's round-to-round range | Held: B/A speedup 0.919, 0.923, 0.931, 0.936 (min) at 16², 64², 128², 256² |
| H1 | the per-Claim span scan costs it (`grid.cell_index` → `position_at` → `content_at` for each Cell of each Claim) | hypothesised → measured | removing the scan recovers most of O2 | C = B with the scan replaced by `black_box(false)` | C ≈ A | Partly held. C/A 0.925, 0.944, 0.946, 0.947. The scan is about a fifth to a quarter of the loss |
| H2 | the grid-sized `Vec<bool>`, the second `Vec<Arc<Claim>>` (grown by `push`, no capacity), and the redistribution walk over every Claim's range cost it | hypothesised → measured | removing them and the scan together recovers all of O2 | D = B without the vector, the unique-Claim `Vec`, or the pass; the `slot_written` field kept but written `false` | D < A | Held. D/A 0.997, 0.993, 0.999, 0.997. C→D is the larger share |
| H3 | `RenderCell` grew and costs bandwidth in the Cell `Vec` | hypothesised → disconfirmed | a size increase, and D slower than A | `size_of::<RenderCell>()` printed by each variant | size unchanged and D ≈ A | Disconfirmed. 40 bytes in A, B and D, since the extra `bool` fits in padding; D ≈ A |
| H4 | runner noise alone | hypothesised → disconfirmed as the whole cause | no local reproduction; attempts disagree | O2; attempt 1 vs 2 | — | Disconfirmed as the whole cause (O2 reproduces). It stays a large error term for CI comparisons (runner variance above) |
| H5 | answering `written` as each Claim is built, from the revision's bytes, costs nothing measurable | hypothesised → measured locally | E ≈ A at every size | E = `ba99abc` with `Claim::written` built in `claims_by_cell` and every Render Frame side structure removed | E slower than A beyond round-to-round range | Held locally. E/A 0.991, 0.993, 0.999, 0.994 (min) and 1.001, 0.997, 1.002, 1.038 (median). CI must confirm it |

Raw local minimum-of-minimums, ns: A 8,556 / 119,640 / 462,000 / 1,848,625, B 9,311 / 129,582 /
496,494 / 1,974,639, C 9,247 / 126,768 / 488,290 / 1,951,808, D 8,586 / 120,445 / 462,519 /
1,854,783, E 8,630 / 120,522 / 462,649 / 1,859,146, at 16², 64², 128², 256².

**Diagnosis.** `ba99abc` added a second whole pass. For every frame it allocated and zeroed a
grid-sized vector and grew a Claim vector without reserving capacity. It walked each Claim's Cells
once to answer `written` through Position conversions, then again to copy that answer. Most of the
added cost is in the allocation and redistribution (H2), and the rest is in the conversion-heavy
scan (H1). Moving the question to where the Claim is built removes both. There, the Claim's range
and the revision's bytes are already in hand, and the scan is a contiguous byte slice that stops at
the first written byte. ADR 0052 records the decision as proposed until this ticket accepts it.
