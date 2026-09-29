# 10 — Settle what the destructor's clock-failure message should say

**What to decide:** `Drop for PlaybackInner` reports `ClockFailure { message: "Playback clock
terminated unexpectedly" }` (`orcvs/src/playback.rs:886-888`). ADR 0041 deleted the clock task, so there
is no longer a clock task to name. Decide what the message should say, and first find out whether
anyone ever sees it.

**The message is wrong.** The engine owns its clock inside the one task; "the Playback clock
terminated" describes a component that no longer exists as a separate thing. For an orderly teardown
it is also wrong in tone — nothing failed.

**Whether it fires on an orderly teardown is unsettled, and the original report stated it as fact.**
`console/src/main.rs:51` runs `eframe::run_native` inside `#[tokio::main]` (`:33`), and `run_native`
owns the `Console`, so the `PlaybackEngine` handles drop while the runtime is still alive with live
workers. The task may well get polled, see `PlaybackEvent::Closed` (`orcvs/src/playback.rs:1472`),
break, and run `inner.stop_contained()` at `:1559` — which leaves `is_playing()` false, so `Drop`
(`:883-885`) returns early and reports nothing at all. It is a scheduling race, not a static property, and which
way it falls is the first thing to establish.

**Even on the `Drop` path the report is probably invisible.** `:886` goes through
`PlaybackInner::report` (`:679-681`), which records into the shared `DiagnosticLog`
(`orcvs/src/playback/diagnostics.rs:231-246`, an `Arc<Mutex<..>>` per ADR 0055). The record always
succeeds, but the console drains that log from the handle, and when the last handle is dropping
there is no console left to drain it. So on this path the wrong message is never user-visible.

The safety action, which is the part that matters, happens either way.

Recorded honestly: wrong message, low impact, firing unconfirmed.

**Status:** needs-triage

- [ ] Whether the destructor's report fires on an orderly console teardown is established, by
      observation rather than by reading — including how often, since it is a race.
- [ ] The message no longer names a clock task that ADR 0041 deleted.
- [ ] If the answer is that nobody can ever receive this report, that is recorded beside the
      destructor, because it changes what the report is for.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and
`console`.

## Comments

Filed from the `playback-actor` review ledger as **CR-11**, downgraded to minor by audit.

The downgrade is the point of the wording above. The original report asserted that a user closing the
console sees "Playback clock terminated unexpectedly"; that has not been observed, two separate
things stand between the teardown and the message, and the issue should not be worked as though it
had been.

`05` covers the other defect in the same destructor.

### Audit at cad296df — 2026-09-29

Re-read against the code; the body is corrected in place.

- The report no longer goes into a channel whose send fails once the receiver is gone. Diagnostics
  are a shared `DiagnosticLog(Arc<Mutex<Retained>>)` (`orcvs/src/playback/diagnostics.rs:231`), so
  the destructor's report is recorded and simply never drained. The conclusion — invisible — holds;
  the mechanism the body quoted does not exist any more.
- The orderly-close path is `PlaybackEvent::Closed => break` at `orcvs/src/playback.rs:1472`, then
  `inner.stop_contained()` at `:1559`; `an_orderly_shutdown_silences_the_output_without_reporting_a_failure`
  (`:4425`) covers it. Line references refreshed: the message 834→886, `Drop`'s early return
  831→883, `report` 668→679.
- The message is still "Playback clock terminated unexpectedly"; no criterion is met.
