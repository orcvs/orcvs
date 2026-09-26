# 28 — Measure the Playback Tick path in CI

**What to build:** CI times and counts the Tick that Playback executes, not only the Tick planned under the Source write lock. Since 07, Playback's Tick goes through `SourceCommander::execute`: a planning snapshot, planning with no lock, and a commit validated against the snapshot's `RevisionId`. The `source_execute_tick*` series call `Source::execute` directly, which is now only the locked fallback, so a regression on the path Playback takes — an extra whole-Grid copy, a lost schedule-cache hit on the snapshot's Language Map — reaches `main` unmeasured. This ticket adds that measurement and records the baseline 08 is judged against.

**Blocked by:** 07 (orcvs/orcvs#154), which introduces the path this measures.

**Status:** resolved

- [x] A single-threaded criterion series executes Ticks through `SourceCommander::execute` over the existing Tick sizes and fixtures, alongside the `Source::execute` series, so the two paths are compared on the same Sources. No thread races an edit into it: it measures the uncontended Tick, deterministically enough for the ratio gate. `SourceCommander::execute` is made reachable from the benchmarks without a test-only item in shipped code.
- [x] The allocation tests record blocks and bytes per Tick on the same path, for at least the shipped Grid, so a copy added to or removed from the Tick is caught as a count rather than a timing.
- [x] `.scratch/benchmarks/spec.md` (or the document CI's benchmark comparison reads) lists the new series, and the ratio gate and path filters cover it.
- [x] The measured baseline — per-Tick time and allocations on both paths — is recorded in a comment on this ticket, for 08 to compare against.

## Comments

**2026-09-26 — origin.** From the architecture review of epic PR 9 (`perf/plan-outside-lock`): the CI Tick series stopped covering the production Tick when planning moved outside the lock. Sequenced before 08 so its storage change lands against a measured baseline.

**2026-09-26 — baseline (orcvs/orcvs#163).** Indicative, Apple M-series, `mise run bench` budget; µs per Tick.

| series | 16x16 | 32x32 | 64x64 | 128x128 |
|---|---|---|---|---|
| `source_execute_tick` (locked) | 8.4 | 28.4 | 111.0 | 454.6 |
| `source_commander_execute_tick` (Playback) | 8.9 | 30.1 | 114.7 | 453.7 |
| `source_execute_tick_edges` | 5.2 | 31.2 | 127.1 | 554.6 |
| `source_commander_execute_tick_edges` | 5.3 | 30.9 | 127.5 | 571.0 |
| `source_execute_tick_portal_inputs` | 12.1 | 48.4 | 194.1 | 812.1 |
| `source_commander_execute_tick_portal_inputs` | 11.9 | 48.2 | 195.5 | 846.8 |

One settled Tick on the shipped 256x256 Grid allocates 35,185 blocks / 11,064,920 bytes locked and 35,186 blocks / 11,130,456 bytes through `SourceCommander::execute`: exactly one extra 64 KiB block, the planning snapshot's copy of the Cells, which 08 removes. Time differs within noise. `SourceCommander::execute` is now `pub`; no existing public entry point runs one Tick synchronously.
