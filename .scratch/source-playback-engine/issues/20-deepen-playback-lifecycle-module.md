# 20 — Deepen the Playback lifecycle module

**What to build:** Put clock ownership, synchronization, cancellation, stale-clock protection, shutdown, and diagnostic observation behind the Playback Engine interface described by ADR-0002.

**Blocked by:** 05 — Run Live Editing through the Playback Engine.

**Status:** resolved

- [x] Playback Engine is a cloneable handle whose interface exposes no lock, cancellation token, Tokio task, generation, `ScheduledTick`, or manual Tick control.
- [x] One fixed-period clock runs at a time; start is idempotent, rejects a zero period or missing Tokio runtime without starting, and executes the first Tick immediately.
- [x] Stop synchronously prevents later Ticks, attempts all-notes-off once, and final-handle drop provides the same safety guarantee.
- [x] Overruns and output failures preserve current Tick behavior and become ordered diagnostics; unexpected clock termination stops Playback and records a diagnostic.
- [x] One atomic observation returns lifecycle state and drains pending diagnostics.
- [x] App and tests use the Playback Engine interface without reproducing its concurrency protocol.
- [x] ADR-0001's Source/Playback seam and both existing adapter seams remain intact.

## Answer

Playback Engine is now a cloneable handle over a private lifecycle implementation. It owns clock spawning, cancellation, generation checks, stop safety, final-handle cleanup, and ordered diagnostics; App starts, stops, and observes Playback without handling concurrency types. `observe` atomically reports playing or stopped and transfers pending diagnostics to App, while private deterministic Tick controls remain available only to the module's focused tests.

## Comments

- Design decisions are recorded in `docs/adr/0002-playback-engine-owns-lifecycle-concurrency.md`.
- `cargo test --workspace`, `cargo check -p console`, and `git diff --check` pass.
- Review regressions cover zero-period start during Playback, final-handle drop during an active Tick, and observing clock failure after output delivery panics.
