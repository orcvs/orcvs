# 16 — Decide whether thirty-three rectangle tests should need a runtime

**What to decide:** Thirty-three console tests became `#[tokio::test]` for no reason of their own —
only because `Orcvs::new` is now fallible and spawns eagerly. Each spins a Tokio runtime and a Playback
Engine task to assert something about where a rectangle is painted. Decide whether that is the price
of ADR 0040 or whether the console's test surface should be able to build a console without a running
engine.

The count, against `origin/main` at `cda0632` rather than against the fork point. 27 in
`console/src/console.rs`, which has 37 plain `#[test]` on main and retains 10; plus 6 in
`console/src/midi.rs`, which has none left. Thirty-three converted, not the "~40" an earlier report
gave and not the 30 this ticket carried before the rebase — main grew three more of the same shape
while the branch was open, and the rebase converted those too.

Thirty-five tests carry a runtime, which is not the same number: `console/src/midi.rs` has 8
`#[tokio::test]`, of which 6 are conversions and 2 are new to this branch. Count conversions when
asking what ADR 0040 charged; count runtimes when asking what the tier now pays for.

It is probably unavoidable and should be said so if it is. `PlaybackEngine::new` spawning eagerly is
one of ADR 0040's stated decisions — "an engine without its task is not one" — and a `Console` holds
an `Orcvs`. Anything that let a layout test skip the engine would be a construction path that exists
for tests, which the repo contract rules out.

What is worth recording either way is the widening itself: a change to the engine's construction can
now turn a geometry test red, and thirty-three tests carry a runtime they do not use. That is a real
cost
and it is currently written down nowhere.

**Status:** needs-triage

**Sources of truth:** ADR 0040 (eager spawn as a decision), ticket 04's Comments (the fallibility
cascade through `Orcvs::new`, `with_source`, `with_output_adapter`, `with_source_and_output_adapter`
and `Console::new`), the repo contract's rule against test-only construction seams.

- [ ] The count is recorded correctly: 33 converted and 35 carrying a runtime, across
      `console/src/console.rs` and `console/src/midi.rs`.
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
Filed at all because it is the clearest measure of what ADR 0040 charged to code that has nothing to
do with playback, and because `18` reports that the same tests are where the tier's timing margin is
tightest.
