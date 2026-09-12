# 03 — Publish observations instead of answering them

**What to build:** Change how the console learns what the engine is doing, while the engine is still behind its mutex. Lifecycle state and the selected destination are published through a `tokio::sync::watch`; ordered diagnostics through a channel the console drains with a non-blocking receive. Nothing awaits, and nothing blocks.

This lands before `04` so that the console-facing change and the ownership change are separate diffs. After this ticket the console no longer asks the engine questions, which is what makes `04` an internal change with no caller-visible surface left to move.

Three call sites need a value inside the egui frame today, and each needs a different answer:

- **`observe`** drains diagnostics through `mem::take` and its result is consumed on the next line, and it also caches lifecycle state that gates Space handling. Split it: state goes in the `watch`, diagnostics come off a channel drained to exhaustion each frame. The atomicity ADR 0002 asks for moves from the lock to the channel.
- **`MidiSelectionHandle::selected_destination_id`** is compared per row to draw the checkmark. It reads the same `watch`.
- **`start` and `retune`** return a `Result` that decides whether `set_bpm` applies the BPM at all. They keep validating on the handle and keep returning synchronously — only the transition they cause becomes a message in `04`.

`MidiSelectionHandle::destinations` needs nothing: it is already cached into `MidiDeviceSelection` and read from there, which is the shape the others are moving to.

The browser constraint is why this is not optional. The main thread has no blocking receive — no `block_on`, no `blocking_recv` — so a reply channel from inside a frame is unavailable there, not merely slow.

**Blocked by:** 02

**Status:** resolved

- [x] Lifecycle state and the selected destination are read from a `watch`, without awaiting.
- [x] Diagnostics are drained from a channel with a non-blocking receive, and still arrive in order, and still arrive exactly once.
- [x] `start` and `retune` still return their `Result` synchronously. The second half of this line —
      `set_bpm` still declines to apply a BPM whose retune failed — was true and covered when it was
      ticked at `92216af` and is neither now. It is left ticked as the record of what this ticket
      delivered, and corrected below; `08` carries the code half.
- [x] The console's behaviour is unchanged: the same diagnostics appear, Space still gates on lifecycle state, the checkmark still marks the same row.
- [x] `PlaybackEngine` is still behind its mutex at the end of this ticket. The ownership change is `04`.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`.

The browser suite is the tier that covers the console's frame, and it is a merge-tier gate rather than a local one — name it on the `Not run` line as deferred to CI, and say that `console/tests/wasm.rs` is where this ticket's risk actually lands.

## Comments

Lifecycle state and the selected destination now go through two `tokio::sync::watch` senders, and
diagnostics through an unbounded `tokio::sync::mpsc`. `PlaybackEngine` is still behind its mutex;
nothing about ownership moved, which is `04`.

`PlaybackObservation` and `PlaybackEngine::observe` are gone, split into `PlaybackEngine::state`
(reads the `watch` receiver the handle holds, no lock and no await) and
`PlaybackEngine::drain_diagnostics` (repeated `try_recv` to exhaustion). `Orcvs::observe_playback`
becomes `Orcvs::drain_playback_diagnostics` — it no longer observes anything, because the cached
`Orcvs::playback_state` field it existed to refresh is deleted and `playing()` reads the published
state at the moment it is asked. `PlaybackInner` keeps no `playing: bool` beside the published
value: `is_playing` reads the sender back through `borrow`, so the fact the engine acts on and the
fact the console gates Space on are the same one.

The selected destination is published from `MidiOutputAdapter`, which owns it and is the only
writer of it — both mutation sites, `select` and the delivery failure that drops the connection,
`send_replace` into the sender. `MidiSelectionHandle` holds a receiver subscribed once at
construction and `selected_destination_id` takes no lock at all now; the "running Orcvs is no
longer available" error comes from the channel closing when the engine that publishes into it is
dropped, which `orcvs/tests/midi_selection_handle.rs` still asserts unchanged.

`start` and `retune` are untouched: same validation on the handle, same synchronous `Result`, and
`set_bpm` still declines to apply a BPM whose retune failed.

`playback::tests::diagnostics_drain_in_order_and_exactly_once` is the new proof, stated over three
recording sites (a refused start, a Tick declined as an Overrun, a refused submission) so that
ordering *between* sites is what is asserted. Three mutations drove it red and were each restored:
draining one item rather than to exhaustion, reporting every diagnostic twice, and reversing the
drained order.

One test needed a real fix rather than a rename.
`clock_failure_remains_observable_after_output_panics` was reading lifecycle state through a lock
the panicking Tick still held, so the lock was doing its synchronisation for it. Reading a
published value takes no lock, so the test now waits for the engine to publish `Stopped` before
draining — the engine records the report before it publishes the stop that report explains, so the
drain cannot miss it. That is a test-synchronisation change, not a behaviour change; the same
diagnostic and the same state are asserted.

`orcvs`'s `tokio` dependency now names the `sync` feature it uses. The feature was already resolved
on transitively through `tokio-util`, so `Cargo.lock` is unchanged and `mise run audit_deps`
passes; what changed is that the requirement is stated where it is used.

Two `#[cfg(test)]` doors went with the split: `PlaybackEngine::diagnostics` (a non-draining peek a
channel cannot offer, and `drain_diagnostics` covers every one of its call sites) and
`PlaybackEngine::is_playing` (now exactly `state() == PlaybackState::Playing`).

`console/tests/wasm.rs` is where this ticket's risk actually lands, and it is a merge-tier gate.
The WASM target was compiled locally — `cargo clippy --workspace --all-targets --target
wasm32-unknown-unknown` covers that test target — but the browser suite itself was not run.

### Correction

The third acceptance line above is no longer backed by anything.
`app::test::failed_tempo_retune_keeps_existing_playback_running` existed at
`92216af:orcvs/src/app.rs:409` and is what made that line true when it was ticked. `ffa2dcc`
— ticket `04` — deleted it, and `04`'s own Comments explain why the failure it staged became
unstageable: `PlaybackStartError::RuntimeUnavailable`, the one retune failure a caller could
produce, moved to construction, leaving `retune` with a single error gated on a zero `Duration`
that `Bpm`'s range cannot yield.

So `Orcvs::set_bpm`'s decline branch is now unreachable *and* unobserved. `04`'s Comments call the
matter "worth a separate look" and no follow-up was filed at the time; it is filed now as `08`,
which asks whether the branch is dead code to delete or a guarantee to re-establish.

The line is not silently unticked. What this ticket delivered was correct at the time it was
delivered, and a later ticket removed its cover — that sequence is the thing worth being able to
read here.
