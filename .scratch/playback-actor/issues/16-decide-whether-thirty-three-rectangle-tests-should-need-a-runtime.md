# 16 — Decide whether thirty-three rectangle tests should need a runtime

**What to decide:** Thirty-three console tests became `#[tokio::test]` for no reason of their own —
only because `Orcvs::new` is now fallible and spawns eagerly. Each spins a Tokio runtime and a Playback
Engine task to assert something about where a rectangle is painted. Decide whether that is the price
of ADR 0041 or whether the console's test surface should be able to build a console without a running
engine.

The count taken at `cda0632` — 33 converted, 35 carrying a runtime, across `console/src/console.rs`
and `console/src/midi.rs` — no longer describes the tree. The console's tests have since moved out of
`console/src/console.rs`, which now holds none, and the surface has grown. At `cad296df` the
`#[tokio::test]` / plain `#[test]` counts are:

| File | `#[tokio::test]` | `#[test]` |
|---|---|---|
| `console/src/console/tests.rs` | 81 | 15 |
| `console/src/console/kittest_tests.rs` | 57 | 3 |
| `console/src/console/storage_tests.rs` | 7 | 0 |
| `console/src/paint.rs` | 38 | 5 |
| `console/src/midi.rs` | 18 | 4 |
| `console/src/marks.rs` | 1 | 0 |

That is 202 console tests carrying a runtime. How many of them need one only because `Orcvs::new`
spawns eagerly, rather than for a reason of their own, has not been re-derived; the recount is part
of this ticket. Count conversions when asking what ADR 0041 charged; count runtimes when asking what
the tier now pays for.

It is probably unavoidable and should be said so if it is. `PlaybackEngine::new` spawning eagerly is
one of ADR 0041's stated decisions — "an engine without its task is not one" — and a `Console` holds
an `Orcvs`. Anything that let a layout test skip the engine would be a construction path that exists
for tests, which the repo contract rules out.

What is worth recording either way is the widening itself: a change to the engine's construction can
now turn a geometry test red, and a large share of the console's runtime-carrying tests carry one
they do not use. That is a real
cost
and it is currently written down nowhere.

**Status:** needs-triage

**Sources of truth:** ADR 0041 (eager spawn as a decision), ticket 04's Comments (the fallibility
cascade through `Orcvs::new`, `with_source`, `with_output_adapter`, `with_source_and_output_adapter`
and `Console::new`), the repo contract's rule against test-only construction seams.

- [ ] The count is recorded against the current layout: how many console tests carry a runtime,
      and how many of those carry it only because `Orcvs::new` spawns the engine.
- [ ] The decision is recorded — accepted cost, or a seam worth finding — and if accepted, the reason
      is written where a later reader of those tests will find it.
- [ ] Nothing proposed introduces a construction path that only tests take.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package console --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package console --locked`. Runtime-per-test cost
shows up under nextest parallelism on the merge tier's macOS run, which is deferred to CI.

## Comments

Filed from the `playback-actor` review ledger as **CR-17**, minor.

The audit corrected the count from "~40" to 30 and the framing from a defect to an accepted cost.
Filed at all because it is the clearest measure of what ADR 0041 charged to code that has nothing to
do with playback, and because `18` reports that the same tests are where the tier's timing margin is
tightest.

### Audit at cad296df — 2026-09-29

The 33/35 count was taken at `cda0632` against `console/src/console.rs` and `console/src/midi.rs`.
`console/src/console.rs` now holds no tests: they moved to `console/src/console/tests.rs`,
`kittest_tests.rs` and `storage_tests.rs`, and `paint.rs`, `midi.rs` and `marks.rs` carry more. The
body now tabulates the current `#[tokio::test]` counts (202 in all) and leaves the conversions-only
figure to be re-derived, rather than keeping a number the tree no longer supports. `Orcvs::new` is
still fallible and spawns eagerly (`orcvs/src/app.rs:171-173`). No rationale for the accepted cost
is written beside the tests yet.
