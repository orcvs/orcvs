# 08 — Give the tick period a type that cannot be zero

**What to decide, then build:** `Orcvs::set_bpm` holds a `Bpm`, a type that cannot be zero, and
converts it to a `Duration`, which can be, one line before handing it to `retune`. Everything
downstream exists to re-establish at runtime the fact the caller already had. Decide whether the
tick period gets a type that carries non-zeroness across the seam, and how far that type reaches.

## The fact is discarded at the call site

`Bpm` (`orcvs/src/opts.rs:63-67`) is a `NonZeroUsize` filtered to `MAX_BPM` (`:10`, `60_000 / 4`,
so 15000). `delay_ms` (`:69-72`) computes `(60000 / bpm) / 4`, which is 1 at the fastest admitted
tempo and never 0 — but it returns a plain `u64`, and `set_bpm` (`orcvs/src/app.rs:205-213`) wraps
that in `Duration::from_millis` at `:207`. `Duration` admits zero. Past that line nothing in the
type says the value came from a `Bpm`.

## What the discard costs

`PlaybackEngine::retune` (`orcvs/src/playback.rs:1031-1037`) re-checks at `:1032` and returns
`PlaybackStartError::ZeroTickPeriod` (`:138`, `Display` arm at `:274`). `set_bpm` then declines the
BPM and calls `report_retune_error` (`:1011-1015`), which emits `PlaybackDiagnostic::RetuneFailure`
(`:125-127`), which `console/src/diagnostics.rs:16` renders into the status line.

None of it can run. `retune` is `pub(crate)`; its only production caller is `app.rs:207`, and its
remaining callers (`playback.rs:1778`, `:2311`, `:2347`, `:2406`, `:2463`) are tests passing
literals. The one other variant a caller could once stage, `RuntimeUnavailable`, moved to
construction in ticket 04: it is raised only by `ClockSpawner::acquire()`, whose sole caller is
`PlaybackEngine::new` (`:976`).

Nor does anything observe it. `app::test::failed_tempo_retune_keeps_existing_playback_running`
existed at `92216af:orcvs/src/app.rs:409` and was deleted by `ffa2dcc`. Ticket 03's third acceptance
box — "`set_bpm` still declines to apply a BPM whose retune failed" — was true when it was ticked
and has had nothing behind it since.

## Why a type, rather than a decision to keep the guard or delete it

Keeping the guard preserves a branch no caller can take and owes a test that has to construct a
`Duration::ZERO` below an entry point that cannot produce one. Deleting the guard removes the branch
but leaves the seam exactly as wide as it was, so the next caller is free to pass zero again and the
failure moves from a declined BPM to whatever `next_scheduled_at` does with it.

A parameter that cannot be zero settles both at once. `retune` gains no zero branch to take, so
`report_retune_error`, `RetuneFailure`, the console arm and `set_bpm`'s decline branch delete
together, and `set_bpm` becomes infallible. No test is owed for a state that cannot be built.
`Bpm` itself may be the type, or a `TickPeriod` newtype over a validated non-zero `Duration` —
`delay_ms` returning `NonZeroU64` is the smaller move and keeps the arithmetic where it is.

## The real decision: how far the type reaches

`start` (`:1049-1055`) is `pub`, takes a `Duration`, and carries the same guard at `:1050`. That one
is reachable from outside the crate and is tested — `a_zero_tick_period_is_refused_and_reported_without_changing_state`
(`:2623`) calls `engine.start(Duration::ZERO)`. Three reaches, in ascending cost:

1. **`retune` only.** `ZeroTickPeriod` stays for `start`, which still needs and has it. Smallest
   change; leaves two spellings of "tick period" in one module.
2. **`retune` and `start`.** `ZeroTickPeriod` leaves `PlaybackStartError` entirely, `PlaybackStartError`
   may reduce to a single variant, and the test at `:2623` goes with the branch it covers. This is a
   public-API change, and the language is pre-release, so it is a decision rather than a breakage.
3. **Into the clock.** `next_scheduled_at` (`:200-218`) keeps its own zero defence at `:206` and a
   `debug_assert` at `:214`, and its doc at `:192-198` justifies itself by pointing back at "`start`
   and `retune` both refuse `ZeroTickPeriod` before any clock is spawned". A non-zero type would make
   that comment a signature.

**Status:** needs-triage

**Sources of truth:** `orcvs/src/opts.rs:63-72` (the non-zero fact and where it is dropped),
`orcvs/src/app.rs:205-213` (the conversion), `orcvs/src/playback.rs:1031-1037` and `:1049-1055` (the
two guards), ticket 04's Comments (the deferral).

- [ ] The tick period crosses the `set_bpm` → `retune` seam as a type that cannot be zero, so
      `retune` has no zero branch left to take.
- [ ] The chosen reach is recorded, as this ticket's answer or an ADR amendment.
- [ ] Whatever that makes unreachable goes with it: `report_retune_error`, `RetuneFailure`, the
      console arm at `console/src/diagnostics.rs:16`, and `set_bpm`'s decline branch — plus
      `ZeroTickPeriod` and its `Display` arm if `start` is included.
- [ ] `set_bpm` no longer carries a branch nothing can take.
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
