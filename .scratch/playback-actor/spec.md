# Give the Playback Engine's state an owner

**Status:** ready-for-agent

## Goal

Move the Playback Engine's state out of `Arc<Mutex<PlaybackInner>>` and into one task that owns it, reached through a cloneable handle that sends messages. Give that task its own clock, so a Tick is one arm of a loop that handles `stop` and `retune` on the other. [ADR 0040](../../docs/adr/0040-the-playback-engine-owns-its-state-in-one-task.md) records the decision and what it keeps from [ADR 0002](../../docs/adr/0002-playback-engine-owns-lifecycle-concurrency.md).

## Why, in one paragraph

`PlaybackInner` holds three lifetimes flat and names one of them. A **run** begins at a `start` that found the engine idle and ends at `stop`; it owns the absolute Tick and the deadline the last Tick was due at, and survives disconnection, destination changes and any number of retunes. A **clock task** begins at `start` *or* `retune`; it owns `generation` and `cancellation`, and a run has several in sequence. A **connection** is orthogonal to both. The tell is `fn begin_run(&mut self) -> (u64, CancellationToken)`: the method named for the run hands back the clock task's identity. Every rule about which field survives which transition lives in a comment at a mutation site, and one of them — `last_output_failure` outliving its run — shipped as a defect and was fixed as `source-playback-engine/22`, whose own acceptance asked that the next such field not be forgotten. Convention is the weakest form of that guarantee. State behind a lock is opaque to the type system; ownership is what makes the ordering expressible, and a task is how a shared handle gets an owner.

## What the facts say, so nobody re-derives them

- **The browser target can run this.** `wasm_bindgen_futures::spawn_local` is already a direct dependency and already spawns the clock through `ClockSpawner`; `tokio::sync` is already enabled transitively via `tokio-util`. What the browser main thread does not have is any blocking receive — no `block_on`, no `blocking_recv` — so a reply channel from inside an egui frame is unavailable, not merely slow.
- **Three call sites genuinely need a value in-frame**: `observe` (drains diagnostics through `mem::take`, consumed on the next line), `start`/`retune` (the `Result` decides whether `set_bpm` applies the BPM at all), and `MidiSelectionHandle::selected_destination_id` (compared per row to draw the checkmark). `destinations` is already cached and deferred.
- **`PlaybackEngine::midi_destinations` and `::selected_midi_destination_id` have no callers anywhere.** Dead public surface.
- **The cost is the tests.** Of 61 tests in `playback.rs`, 34 drive only the `#[cfg(test)]` doors, 9 mix doors with the handle, 10 use the handle only, and 8 touch pure helpers an actor never reaches. `.scratch/playback-clock/spec.md` deliberately put reshaping that door out of scope, because those tests "exercise Tick execution — ownership, expiry, delivery order — and driving them through a clock would make them slower without making them truer". That reasoning is why `02` comes before `04`: those tests do not get driven through a clock, they stop going through the engine at all.
- **The five `compile_fail` doctests on `MidiSelectionHandle` will not protect this seam.** They assert method *absence*, so they stay green through any rename or signature change.

## Decisions

- The whole of `PlaybackInner` moves into the task. A half-measure leaves the lock, and the lock is the problem.
- One task, not two. The engine waits on its message channel and its next deadline together. `generation`, `CancellationToken` and `ClockRunGuard` are deleted rather than renamed — with no second party, there is nothing to be stale relative to.
- Lifecycle state and the selected destination are published through a `watch`; ordered diagnostics through a channel drained with a non-blocking receive. Neither awaits.
- `start` and `retune` keep validating on the handle and return their `Result` synchronously; only the transition becomes a message. `PlaybackStartError::RuntimeUnavailable` moves to construction, because that is where a runtime is now required.
- `PlaybackEngine::new` becomes fallible and spawns eagerly. An engine without its task is not one.
- The `Arc<AtomicUsize>` handle count goes. Dropping the last sender closes the channel, the task sees the close, runs the safety action and exits.
- `stop` keeps ADR 0002's synchronous guarantee through a one-way flag set before the message is sent and read before each Tick executes. It is shared state, admitted deliberately, and named for what it carries — a request to stop — never for whether the engine is playing.

## Delivery order

Derived from each issue's `Status:` and `Blocked by:` lines, not authored here.

## Required behavior

Every guarantee ADR 0002 states is kept, including that further Ticks are prevented before `stop` returns. The Source/Playback seam and ADR 0037's Tick Grid rule are unchanged. A run still has one fixed Tick period at a time, still executes its first Tick immediately, and still turns output failures and Overruns into diagnostics without rolling back Source writes or stopping later Ticks.

The output adapter seam is the one exception, and this line used to claim otherwise. `OutputAdapter` gained a defaulted `published_destinations` (`orcvs/src/playback.rs:88-90`), overridden by `MidiOutputAdapter` (`orcvs/src/midi.rs:309-311`) and absent at `25c8de8`. Moving the adapter into the task leaves no synchronous path from `MidiSelectionHandle::destinations()`/`select()` to it, and the browser frame has no blocking receive to await a reply on, so what those asked for is published instead — and a generic `PlaybackEngine::new` needs the subscription while the adapter is still in hand. `04`'s Comments record the deviation; ADR 0040 records the same correction and the open question of whether the subscription belongs on the general trait at all.

## Out of scope

- The Tick pipeline in `orcvs/src/source/`. It is a pipeline, not a state machine: nothing persists between its stages and nothing receives events there. Its problems are interface problems and are tracked elsewhere.
- `source-playback-engine/23`, which proposed giving the shared state better types behind the lock. ADR 0040 records why that treats the symptom; this effort absorbs it and `23` should be closed against this spec rather than built.
