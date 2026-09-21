# 04 — Prove Paint recovered without making Source revisions slower

**What to build:** Prove on the authoritative benchmark runner that `03` removes the dominant Paint
cost without transferring it to Source revision reads, Source edits, Tick execution, or Render
Frame derivation. Record the result and reconcile the decisions and deferred cache work against the
measured implementation.

**Blocked by:** 03 — Remove the Paint claim cache without caching Paint on the Language Map.

**Status:** resolved

**Tags:** release/v1

- [x] The pull-request Benchmark workflow compares the candidate with the rebased `origin/main` on
      the same runner and records every `paint_derive`, `paint_background_runs`,
      `source_read_revision`, `source_edit_rebuild_valid`, `source_edit_rebuild_invalid`,
      `source_execute_tick`, and `source_render_frame` result.
- [x] Both fitted and culled `paint_derive` series materially improve in the direction the profile
      in `01` predicts: the pointer-keyed cache and its growth cost are absent from the measured
      path.
- [x] No Source revision read, valid or invalid edit rebuild, Tick execution, Render Frame, or
      background-run series regresses beyond ordinary runner noise. Any apparent regression is
      rerun and resolved rather than accepted because the Paint series improved.
- [x] ADR 0050 is amended or superseded to record the measured seam: the Render Frame carries the
      once-derived written answer, while the Language Map remains authoritative for the parser's
      claim and does not retain whole-Grid Paint views.
- [x] `paint-cell-cost/02` records that its whole-Grid Language Map implementation was rejected by
      the Source benchmark regressions, without erasing the implementation and review history.
- [x] The remaining claim-cache criterion in `syntax-highlighting/07` is checked only after the
      benchmark comparison proves the cache is gone without transferring unacceptable cost.
- [x] The final record names the candidate SHA, baseline SHA, workflow run, exact measurements, and
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

## Accepted: `f394d81`

**Decision: accepted.** Paint recovered 3.3–5.5x and nothing this ticket protects regressed. The one
apparent regression, `paint_background_runs`, was rerun and reversed direction, which is what a
runner difference does and not what a cost does.

- Baseline: `3650c7d8c463cf1e48fa7d37c8e84b408ad234be` (`origin/main`), the point `main`'s run
  [35503614546](https://github.com/orcvs/orcvs/actions/runs/35503614546) stored on `gh-pages`.
- Candidate: `f394d81` — `1f70fa1` (the implementation) with this record on top.
- Workflow: [35548734385](https://github.com/orcvs/orcvs/actions/runs/35548734385), job
  `pull-request`, attempts 1 and 2. Attempt 2 is the rerun this ticket requires.

Speedup is `baseline / candidate`; above 1.000 is faster. **Control** is the geometric mean over the
18 `source_edit_rebuild_*` and `source_execute_tick*` points, whose code neither commit touches, so
it measures the runner rather than the change. Attempt 1's runner matched the baseline's
(control 0.997, and `source_read_revision` reproduced the baseline to the nanosecond at four of five
sizes). Attempt 2's was faster (control 1.218), so its raw numbers are reported beside the
normalised ones rather than instead of them.

| benchmark | baseline | attempt 1 | speedup | normalised | attempt 2 | speedup | normalised |
|---|---:|---:|---:|---:|---:|---:|---:|
| `source_render_frame/16x16` | 14,410 (154) | 14,019 (211) | 1.028 | 1.031 | 11,107 (14) | 1.297 | 1.065 |
| `source_render_frame/32x32` | 56,893 (4,310) | 54,481 (677) | 1.044 | 1.048 | 42,113 (28) | 1.351 | 1.109 |
| `source_render_frame/64x64` | 239,643 (2,543) | 223,725 (2,921) | 1.071 | 1.074 | 176,872 (1,127) | 1.355 | 1.112 |
| `source_render_frame/128x128` | 922,920 (9,006) | 898,041 (10,337) | 1.028 | 1.031 | 703,606 (2,729) | 1.312 | 1.077 |
| `source_render_frame/256x256` | 3,782,733 (141,749) | 3,627,207 (49,632) | 1.043 | 1.046 | 3,493,644 (193,927) | 1.083 | 0.889 |
| `paint_derive/fitted/16x16` | 11,794 (17) | 2,842 (11) | 4.150 | — | 2,133 (3) | 5.529 | — |
| `paint_derive/fitted/32x32` | 47,208 (224) | 12,335 (40) | 3.827 | — | 9,101 (18) | 5.187 | — |
| `paint_derive/fitted/64x64` | 192,336 (507) | 52,603 (609) | 3.656 | — | 36,261 (55) | 5.304 | — |
| `paint_derive/fitted/128x128` | 785,236 (45,672) | 228,457 (2,727) | 3.437 | — | 158,092 (343) | 4.967 | — |
| `paint_derive/fitted/256x256` | 3,205,954 (6,152) | 979,993 (972) | 3.271 | — | 667,079 (1,479) | 4.806 | — |
| `paint_derive/culled/16x16` | 11,702 (23) | 2,820 (9) | 4.150 | — | 2,132 (2) | 5.489 | — |
| `paint_derive/culled/32x32` | 11,740 (27) | 2,832 (38) | 4.145 | — | 2,148 (4) | 5.466 | — |
| `paint_derive/culled/64x64` | 11,750 (38) | 2,898 (13) | 4.055 | — | 2,182 (2) | 5.385 | — |
| `paint_derive/culled/128x128` | 11,864 (33) | 2,918 (15) | 4.066 | — | 2,274 (28) | 5.217 | — |
| `paint_derive/culled/256x256` | 11,912 (44) | 2,947 (11) | 4.042 | — | 2,212 (11) | 5.385 | — |
| `paint_background_runs/fitted/16x16` | 691 (1) | 716 (4) | 0.965 | 0.968 | 503 (32) | 1.374 | 1.128 |
| `paint_background_runs/fitted/32x32` | 2,072 (8) | 2,220 (5) | 0.933 | 0.936 | 1,639 (35) | 1.264 | 1.038 |
| `paint_background_runs/fitted/64x64` | 7,141 (20) | 7,546 (32) | 0.946 | 0.949 | 5,706 (16) | 1.251 | 1.028 |
| `paint_background_runs/fitted/128x128` | 27,718 (48) | 29,017 (60) | 0.955 | 0.958 | 21,957 (62) | 1.262 | 1.036 |
| `paint_background_runs/fitted/256x256` | 120,255 (362) | 123,264 (179) | 0.976 | 0.979 | 98,835 (242) | 1.217 | 0.999 |
| `paint_background_runs/culled/16x16` | 697 (2) | 725 (4) | 0.961 | 0.964 | 517 (1) | 1.348 | 1.107 |
| `paint_background_runs/culled/32x32` | 661 (3) | 703 (1) | 0.940 | 0.943 | 502 (1) | 1.317 | 1.081 |
| `paint_background_runs/culled/64x64` | 624 (1) | 664 (33) | 0.940 | 0.943 | 467 (1) | 1.336 | 1.097 |
| `paint_background_runs/culled/128x128` | 665 (3) | 694 (1) | 0.958 | 0.961 | 494 (0) | 1.346 | 1.105 |
| `paint_background_runs/culled/256x256` | 660 (1) | 698 (2) | 0.946 | 0.949 | 496 (0) | 1.331 | 1.093 |
| `source_read_revision/16x16` | 34 (0) | 34 (0) | 1.000 | — | 41 (0) | 0.829 | — |
| `source_read_revision/32x32` | 42 (0) | 42 (0) | 1.000 | — | 42 (0) | 1.000 | — |
| `source_read_revision/64x64` | 102 (0) | 102 (0) | 1.000 | — | 86 (0) | 1.186 | — |
| `source_read_revision/128x128` | 249 (0) | 249 (1) | 1.000 | — | 212 (0) | 1.175 | — |
| `source_read_revision/256x256` | 1,373 (1) | 1,373 (1) | 1.000 | — | 1,188 (1) | 1.156 | — |
| `source_edit_rebuild_valid/16x16` | 7,103 (28) | 6,909 (62) | 1.028 | — | 6,070 (10) | 1.170 | — |
| `source_edit_rebuild_valid/32x32` | 28,462 (2,400) | 28,357 (125) | 1.004 | — | 23,659 (106) | 1.203 | — |
| `source_edit_rebuild_valid/64x64` | 115,234 (561) | 113,269 (648) | 1.017 | — | 97,793 (357) | 1.178 | — |
| `source_edit_rebuild_invalid/16x16` | 7,414 (46) | 7,236 (44) | 1.025 | — | 6,280 (19) | 1.181 | — |
| `source_edit_rebuild_invalid/32x32` | 28,630 (166) | 28,360 (90) | 1.010 | — | 23,774 (76) | 1.204 | — |
| `source_edit_rebuild_invalid/64x64` | 115,779 (363) | 113,267 (321) | 1.022 | — | 98,377 (246) | 1.177 | — |
| `source_execute_tick/16x16` | 21,980 (96) | 22,128 (47) | 0.993 | — | 17,612 (449) | 1.248 | — |
| `source_execute_tick/32x32` | 81,645 (211) | 82,584 (255) | 0.989 | — | 64,323 (141) | 1.269 | — |
| `source_execute_tick/64x64` | 326,845 (1,658) | 329,669 (829) | 0.991 | — | 265,662 (6,984) | 1.230 | — |
| `source_execute_tick/128x128` | 1,374,008 (2,434) | 1,376,620 (5,683) | 0.998 | — | 1,118,287 (11,126) | 1.229 | — |
| `source_execute_tick_edges/16x16` | 14,305 (40) | 14,483 (90) | 0.988 | — | 11,574 (123) | 1.236 | — |
| `source_execute_tick_edges/32x32` | 84,293 (202) | 85,157 (208) | 0.990 | — | 66,482 (166) | 1.268 | — |
| `source_execute_tick_edges/64x64` | 360,903 (844) | 365,955 (871) | 0.986 | — | 286,979 (1,406) | 1.258 | — |
| `source_execute_tick_edges/128x128` | 1,532,063 (5,447) | 1,547,688 (8,588) | 0.990 | — | 1,251,971 (18,047) | 1.224 | — |
| `source_execute_tick_portal_inputs/16x16` | 27,839 (75) | 28,651 (50) | 0.972 | — | 22,501 (418) | 1.237 | — |
| `source_execute_tick_portal_inputs/32x32` | 114,481 (292) | 117,825 (249) | 0.972 | — | 93,704 (313) | 1.222 | — |
| `source_execute_tick_portal_inputs/64x64` | 474,931 (1,215) | 478,383 (1,183) | 0.993 | — | 393,897 (3,987) | 1.206 | — |
| `source_execute_tick_portal_inputs/128x128` | 2,007,097 (5,472) | 2,048,253 (6,421) | 0.980 | — | 1,684,991 (22,036) | 1.191 | — |

**Allocations.** All 21 series are identical to the baseline in both attempts, including
`orcvs write one cell 64x64 populated blocks` 1,110 and `bytes` 647,744, and
`lang render frame re-read fixture blocks` 24 and `bytes` 48. Nothing this change added allocates
on a Source write path, and the Render Frame lost a grid-sized `Vec<bool>` and a Claim `Vec` that
the rejected candidate allocated per frame — neither is counted by these series, which measure
Source writes rather than frames.

**The rerun, and what it settled.** Attempt 1 showed `paint_background_runs` at 0.933–0.976 across
every size. `Paint::background_runs` and `CellPaint` are the same code and the same layout in
`3650c7d` and here — the diff between them touches `Paint::derive_with_colours` and tests only — so
either the fold's input changed or the runner did. The rerun answers it: attempt 2 read 1.217–1.374
raw, 0.999–1.128 normalised, on the same code. A cost does not reverse sign between two runs of one
commit. The rejected candidate's run is further evidence for the same reading: it measured
`paint_background_runs/culled` at 0.977–1.135 while this one measured 0.940–0.961, on a series
neither commit changes. This is recorded as resolved by rerun, not as accepted noise: the direction
did not survive, which is the test this ticket asks for.

**Uncertainty that remains.**

- Every comparison is against one stored baseline point, measured in a different job on a different
  day. The control normalisation bounds that, it does not remove it.
- `source_render_frame/256x256` is the widest point in the group: the baseline's own spread is
  ±141,749 (3.7%) and attempt 2's is ±193,927 (5.5%). It is faster than baseline raw in both
  attempts (1.043, 1.083), and its attempt-2 normalised 0.889 is the one figure below 1.000 in the
  Render Frame row. That normalisation assumes the runner scales uniformly across benchmarks, which
  the control set cannot show for a shape it has no point at — every control benchmark is 128x128 or
  smaller. Attempt 1, whose runner matched the baseline's and needed no normalisation, reads 1.043.
- Paint's improvement is large enough that runner class does not change its sign: the smallest
  measurement of it is 3.271x.
- The local ablation behind the diagnosis ran on an Apple M2 under load. It is diagnostic only.

**Decision: keep.** `ba99abc` is rejected and `f394d81` is accepted. ADR 0052 moves to accepted.
