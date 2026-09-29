# 08 — Give the tick period a type that cannot be zero

**What to decide, then build:** `Orcvs::set_bpm` holds a `Bpm`, a type that cannot be zero, and
converts it to a `Duration`, which can be, in the same expression that hands it to `retune`.
`retune`'s zero branch exists to re-establish at runtime the fact the caller already had. Decide
whether the tick period gets a type that carries non-zeroness across the seam, and how far that type
reaches.

## The fact is discarded at the call site

`Bpm` (`orcvs/src/opts.rs:45`, constructor `:73-77`) is a `NonZeroUsize` filtered to `MAX_BPM`
(`:21`, `QUARTER_MINUTE_NANOS / 1_000_000`, so 15000). `Bpm::tick_period` (`:91-94`) divides a
quarter minute in nanoseconds by the BPM, rounding halves up, which is 1 ms at the fastest admitted
tempo and never 0 — but it returns a plain `Duration`, and `set_bpm` (`orcvs/src/app.rs:379-387`)
passes `bpm.tick_period()` straight to `retune` at `:381`. `Duration` admits zero. Past that call
nothing in the type says the value came from a `Bpm`.

## What the discard costs

`PlaybackEngine::retune` (`orcvs/src/playback.rs:1183-1191`) re-checks at `:1184` and returns
`PlaybackStartError::ZeroTickPeriod` (`:230`, `Display` arm at `:391`). On any retune error
`set_bpm` declines the BPM and calls `report_retune_error` (`:1159-1163`), which emits
`PlaybackDiagnostic::RetuneFailure` (`:128-130`), which `console/src/diagnostics.rs:19` renders into
the status line.

Only the zero branch is dead. `retune` is `pub(crate)`; its only production caller is `app.rs:381`,
and a `Bpm` cannot produce a zero period. But `retune` has two other failures:
`UnschedulableTickPeriod` (`:1187-1189`, when `ClockInstant::now() + period` overflows — not
reachable from a `Bpm`'s 1 ms–15 s range in practice) and `EngineUnavailable` (`:1190`, via
`From<Unavailable>` at `:549-553`), which the queue returns once the Playback task has ended. That
one is reachable from `set_bpm` while playback is requested, so `RetuneFailure`,
`report_retune_error`, the console arm and `set_bpm`'s decline branch are live code, not dead
weight. `RuntimeUnavailable` (`:231`) is raised only at construction.

Nor does anything observe the decline at the app level.
`app::test::failed_tempo_retune_keeps_existing_playback_running` existed at
`92216af:orcvs/src/app.rs:409` and was deleted by `ffa2dcc`. The only remaining exercise is the
engine-level `report_retune_error` call in a test at `playback.rs:4235`; no test drives `set_bpm`
into its decline branch. Ticket 03's third acceptance box — "`set_bpm` still declines to apply a BPM
whose retune failed" — has had no test behind it since.

## Why a type, rather than a decision to keep the guard or delete it

Keeping the guard preserves a branch no caller can take and owes a test that has to construct a
`Duration::ZERO` below an entry point that cannot produce one. Deleting the guard removes the branch
but leaves the seam exactly as wide as it was, so the next caller is free to pass zero again and the
failure moves from a declined BPM to whatever `next_scheduled_at` does with it.

A parameter that cannot be zero removes the zero branch without widening anything: `retune` has
no zero case to take, and no test is owed for a state that cannot be built. It does not make
`set_bpm` infallible — `EngineUnavailable` still reaches the decline branch, so that branch,
`report_retune_error`, `RetuneFailure` and the console arm stay, and the decline branch owes the
app-level test that `ffa2dcc` deleted. `Bpm` itself may be the type, or a `TickPeriod` newtype over a
validated non-zero `Duration` — `Bpm::tick_period` returning that newtype is the smaller move and
keeps the arithmetic where it is.

## The real decision: how far the type reaches

`start` (`:1200-1210`) is `pub`, takes a `Duration`, and carries the same guard at `:1201`. That one
is reachable from outside the crate and is tested — `a_zero_tick_period_is_refused_and_reported_without_changing_state`
(`:3826`) calls `engine.start(Duration::ZERO)`. Three reaches, in ascending cost:

1. **`retune` only.** `ZeroTickPeriod` stays for `start`, which still needs and has it. Smallest
   change; leaves two spellings of "tick period" in one module.
2. **`retune` and `start`.** `ZeroTickPeriod` leaves `PlaybackStartError` entirely, `PlaybackStartError`
   keeps its other variants, and the test at `:3826` goes with the branch it covers. This is a
   public-API change, and the language is pre-release, so it is a decision rather than a breakage.
3. **Into the clock.** `next_scheduled_at` (`:317-339`) keeps its own zero defence at `:323` and a
   `debug_assert` at `:331`, and its doc at `:312-316` justifies itself by pointing back at "`start`
   and `retune` both refuse `ZeroTickPeriod` before any clock is spawned". A non-zero type would make
   that comment a signature.

**Status:** needs-triage

**Sources of truth:** `orcvs/src/opts.rs:73-94` (the non-zero fact and where it is dropped),
`orcvs/src/app.rs:379-387` (the conversion), `orcvs/src/playback.rs:1183-1191` and `:1200-1210` (the
two guards), ticket 04's Comments (the deferral).

- [ ] The tick period crosses the `set_bpm` → `retune` seam as a type that cannot be zero, so
      `retune` has no zero branch left to take.
- [ ] The chosen reach is recorded, as this ticket's answer or an ADR amendment.
- [ ] Only what that makes unreachable goes with it: `retune`'s zero guard, plus `ZeroTickPeriod`
      and its `Display` arm if `start` is included. `report_retune_error`, `RetuneFailure`, the
      console arm at `console/src/diagnostics.rs:19` and `set_bpm`'s decline branch stay, because
      `EngineUnavailable` still reaches them.
- [ ] `set_bpm`'s decline branch has an app-level test driving it through `EngineUnavailable`.
- [ ] Ticket 03's third acceptance line reflects whichever answer is chosen.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and
`console`. Reach 2 or 3 shrinks the public API, so `cargo test --workspace --doc --locked` too.

## Comments

Filed from the `playback-actor` review ledger as **CR-08** (major). The record half — ticket 03's
acceptance box — is corrected in that ticket and points here.

Rewritten after review. The first draft framed this as keep-the-guard versus delete-it, on the
strength of "`retune` is public and takes a `Duration`, so the guard is its stated contract".
`retune` is `pub(crate)` (`playback.rs:1031`), so no out-of-tree caller exists and that argument for
keeping it does not. Once that is corrected the parameter type is the defect, and both original
answers are worse than changing it. The public-API half of the question is real, but it belongs to
`start`, not `retune`.

### Audit at cad296df — 2026-09-29

Re-read against the code; the body is corrected in place.

- `delay_ms` no longer exists. `Bpm::tick_period` (`orcvs/src/opts.rs:91-94`, `15780be0`) returns a
  nanosecond-rounded `Duration`, and `set_bpm` (`orcvs/src/app.rs:379-387`) passes it to `retune`
  directly.
- The claim that `retune` could fail only with `ZeroTickPeriod`, so its whole failure path was dead,
  was false. `retune` (`orcvs/src/playback.rs:1183-1191`) also returns `UnschedulableTickPeriod` and,
  through `From<Unavailable>` (`:549-553`), `EngineUnavailable` once the task has ended. The
  deletion cascade is narrowed to the zero guard; the decline branch and the diagnostic stay, and a
  criterion now asks for their app-level test.
- Every line reference moved: `retune` 1031→1183, `start` 1049→1200, the enum 138→230, `Display`
  274→391, `report_retune_error` 1011→1159, `next_scheduled_at` 200→317, the zero-start test
  2623→3826, and the console arm `diagnostics.rs:16`→`:19`.
