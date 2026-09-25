# 01 — Derive the Tick period from BPM without truncation

**What to build:** The Tick period is derived from BPM at sub-millisecond precision, so Playback keeps tempo with external MIDI gear. Today `Bpm::delay_ms` (`orcvs/src/opts.rs`) computes the period in whole milliseconds with two truncating divisions: 130 BPM ticks at 115 ms instead of 115.38 ms (about 0.33% fast, drifting against a synced device), and every BPM from 7501 to the maximum (15000) collapses to a 1 ms Tick. Its callers turn it into a `Duration` with `Duration::from_millis` in `Orcvs::set_bpm` and `Orcvs::play` (`orcvs/src/app.rs`), and a console test builds the period the same way.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The Tick period for any accepted BPM equals one quarter of a minute (a sixteenth note) divided by that BPM — 125 ms at 120 BPM — to at least microsecond precision.
- [x] The rounding policy and error bound against the mathematical period are recorded. A fixed nanosecond `Duration` may introduce at most one nanosecond of period error per Tick; tests check the resulting bound over a long run. If non-accumulating tempo error is required instead, deadlines carry a rational remainder and tests establish that stronger bound.
- [x] Scheduler lateness does not compound into phase drift: `next_scheduled_at` and `first_retuned_tick_at` retain their deadline-based scheduling and skip behavior. Tests distinguish this property from period-rounding error.
- [x] Every caller, including the console test that builds the period from `delay_ms`, takes the precision-preserving period; the doc comment on the drift rule stops describing whole-millisecond periods.
- [x] Tests cover BPMs that do not divide evenly (for example 130 and 9000) and the maximum BPM, not only 20 and 120.
- [x] The displayed-beat rule is unchanged.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Bug still present. The earlier criterion said "one sixteenth of a minute", which is wrong; corrected above. Call sites named.

**2026-09-25 — acceptance-criteria review against `199c3331`.** Period rounding and scheduler drift are separate contracts. Adding a rounded Duration repeatedly cannot promise zero accumulated error against mathematical tempo.

**2026-09-25 — implementation.** Implemented in orcvs/orcvs#145 (epic PR 2): `Bpm::tick_period` rounds a quarter minute over the BPM to the nearest nanosecond (≤0.5 ns per Tick); policy in ADR 0037. Resolve on merge.

**2026-09-25 — resolved.** Merged in orcvs/orcvs#145 (`51cf1c11`); every criterion verified on `main`.
