# 04 — Move the engine's state into the task that owns it

**What to build:** [ADR 0041](../../../docs/adr/0041-the-playback-engine-owns-its-state-in-one-task.md), implemented. `PlaybackInner` stops living behind `Arc<Mutex<..>>` and becomes the state of one task. `PlaybackEngine` becomes a cloneable handle that sends messages. That task owns its clock: it waits on its message channel and its next deadline together, so a Tick is one arm of a loop that processes `stop` and `retune` on the other.

**What gets deleted, not renamed.** `generation`, `CancellationToken` and `ClockRunGuard` all exist because a second task calls into shared state. `generation` makes a Tick from a retired clock decline; `cancellation` wakes a retired clock out of its sleep; `ClockRunGuard` notices when the clock task dies without finishing. With one task there is no second party to be stale relative to, nothing sleeping that must be woken, and no other task whose death must be detected. A retune recomputes the deadline the loop waits on and that is the whole of it.

The `Arc<AtomicUsize>` handle count goes too. Dropping the last sender closes the channel; the task sees the close, runs the safety action, and exits — the same guarantee, structurally rather than arithmetically. The two tests that assert it survive as behavioural assertions.

`PlaybackEngine::new` becomes fallible and spawns eagerly, which is where `PlaybackStartError::RuntimeUnavailable` moves to. The native-only test that hand-builds a `Runtime` to stage that failure moves with it.

**`stop` keeps its synchronous guarantee, and this is the part to get right.** ADR 0002 requires that further Ticks are prevented before `stop` returns, and sending a message does not do that — `send` returns once the message is queued and the task may be mid-Tick. A one-way flag, set by the handle before the message goes and read by the task before it executes each Tick, restores it. Name it for what it carries: a request to stop. It is not "playing", it is not the lifecycle state, and nothing may read it to decide which state the engine is in. ADR 0041 admits this one piece of shared state deliberately and says why; a later reader collapsing it into the lifecycle state would re-admit exactly what this effort removes.

**Spawning.** `ClockSpawner` already has the cfg seam this needs: native takes a `tokio::runtime::Handle`, the browser uses `wasm_bindgen_futures::spawn_local` with no `Send` bound. The actor task spawns the same way. Note the asymmetry already in the tree — the lifecycle methods sit in an `impl` block carrying `Send` on both targets even though the browser spawn does not need it.

**Blocked by:** 03

**Status:** resolved

- [x] `PlaybackInner` is owned by one task. No mutex, and the handle exposes no lock, token, task or generation.
- [x] The task owns its clock; there is no separate clock task.
- [x] `generation`, `CancellationToken`, `ClockRunGuard` and the handle count are gone.
- [x] Dropping the last handle closes the channel, and the task runs the safety action before exiting.
- [x] `stop` prevents further Ticks before it returns, proved by a test, through a flag named for the request it carries.
- [x] `new` is fallible and spawns eagerly; `RuntimeUnavailable` is reported at construction.
- [x] Every guarantee ADR 0002 states still holds, and ADR 0037's Tick Grid rule is untouched — a retune still anchors on the deadline the last executed Tick was due at.
- [x] `source-playback-engine/23` is closed against ADR 0041 rather than built.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`. Public API changes, so `cargo test --workspace --doc --locked` — and note that the five `compile_fail` doctests on `MidiSelectionHandle` assert method *absence*, so they will stay green through this whether or not the seam survives. They are not evidence here.

This is the ticket whose risk lands in the browser suite and in the concurrency tests, neither of which a local run covers: `mise run test_wasm` and the merge tier are deferred to CI, and this ticket should say so rather than imply the local gates settled it.

## Comments

`PlaybackInner` is the state of one task. `PlaybackEngine` is a handle holding an
`mpsc::UnboundedSender<PlaybackCommand<A>>`, the readers of what the engine publishes, and the
one `AtomicBool` ADR 0041 admits. There is no lock over the state, no `Weak`, no token, no task
handle and no generation anywhere in the module.

**The three mechanisms are deleted, not renamed.** `generation`, `CancellationToken` and
`ClockRunGuard` are gone, and `tokio_util::sync::CancellationToken` is no longer imported.
`execute_tick` takes a `TickTiming` and nothing else: it no longer asks whether a run is live or
whether the Tick belongs to the current clock, because the grid the deadline came from exists
exactly while a run does and belongs to the same task. A retune recomputes the deadline the loop
waits on — `clock = Some(TickClock::retuned(..))` — and that is the whole of it.

**One task, one loop.** `run_engine` waits on `next_playback_event`, which is the message channel
and the next deadline together, `biased` so a message wins a tie. A first deadline already reached
is answered without awaiting, which is what keeps a run's first Tick inside the turn the run began
in; that mattered enough to the browser that `03`'s predecessor wrote a paragraph about it, and
the rule is unchanged.

**The handle count is gone.** Dropping the last `PlaybackEngine` closes the queue; the task sees
the close, stops the run — which sends the safety action — and exits.
`dropping_the_final_handle_stops_playback_safely` and
`dropping_the_final_handle_during_a_tick_completes_playback_safety` survive as behavioural
assertions. The second one no longer asserts that the drop *blocks* until the Tick finishes: there
is no lock left for it to block on, and a `stop` that held a console frame behind a device
submission is a cost ADR 0041 removes. What it asserts now is that the drop returns without
waiting and the safety action still arrives, exactly once.

**`stop` keeps its synchronous guarantee.** `PlaybackEngine::stop_requested: Arc<AtomicBool>` is
set by the handle with `Release` before the `Stop` message is sent and read by the task with
`Acquire` immediately before each Tick would execute, with nothing between the load and the
execution. A load that begins after the store completed observes it — that is the atomic's own
coherence — so no Tick begins executing after `stop` has returned. The field is named for the
request it carries, is documented as carrying one fact in one direction, and nothing reads it to
decide which state the engine is in; the lifecycle state is the `watch` and belongs to the task.
A requested stop also drops the grid, so nothing spins declining deadlines while the message
behind the request arrives, and the `Stop` arm is the only place the request is cleared.

**Construction.** `PlaybackEngine::new` is fallible and spawns eagerly;
`PlaybackStartError::RuntimeUnavailable` is answered there. That makes `Orcvs::new`,
`with_source`, `with_output_adapter`, `with_source_and_output_adapter` and `Console::new`
fallible too, which is most of the diff outside `playback.rs`: every test and doctest that builds
a running Orcvs now does so on a runtime. The native-only test that hand-built a `Runtime` to
stage the failure moved with it and is now
`app::test::a_running_orcvs_is_refused_when_there_is_no_runtime_to_run_on`.

**What the ticket did not name, and had to be decided.** The adapter moves into the task with the
rest of the state, so `MidiSelectionHandle` can no longer reach it to ask it anything —
`destinations()` and `select()` were still questions. They are published now, the way ADR 0041
publishes everything else: `MidiOutputAdapter` owns one `watch::Sender<MidiDestinations>` carrying
both the destinations the last discovery found (or the failure it reported) and the one the
adapter is connected to; `OutputAdapter::published_destinations` is the one defaulted trait method
that lets a generic `PlaybackEngine::new` take that subscription while the adapter is still in
hand. `MidiSelectionHandle::refresh_destinations` asks, `destinations` reads the published answer,
and `select` asks — a device that refuses a connection is now reported on the diagnostics stream
rather than returned, which is the stream the console already turns into its status line. The
console reads the published list every frame the menu is drawn, so a refresh answered by the
engine appears on the next frame with nothing for the user to click twice. The handle still holds
a weak sender, so every method still answers "running Orcvs is no longer available" once the last
`PlaybackEngine` is gone, which `orcvs/tests/midi_selection_handle.rs` still pins.

**`MidiSelectionHandle`'s `compile_fail` doctests are not evidence and were not treated as such.**
They assert method *absence*, so they stayed green through all of this; the one thing done to them
was to make them `unwrap()` the now-fallible `Orcvs::new`, so that each still fails to compile for
the reason it names rather than for the Result. The evidence that the seam survived is elsewhere:
`MidiSelectionHandle` holds a weak sender and a `watch::Receiver` and has no field or method that
could reach lifecycle control; the workspace compiles for both targets with `-D warnings`; and
`orcvs/tests/midi_selection_handle.rs` drives the handle from outside the crate.

**Tests.** The 34 tests that drove `#[cfg(test)]` doors on the handle now drive `PlaybackInner`
directly through a `HandDrivenRun` in the test module, and the four doors —
`clock_tick`, `activate_for_test`, `current_tick`, `holds_note_ownership` — are gone from shipped
code entirely. `PlaybackInner::new` is shipped and is what both `PlaybackEngine::new` and the
tests build state with, so nothing test-only was cut into the engine.

Two tests were dropped rather than rewritten.
`a_tick_the_engine_declines_consumes_no_absolute_tick` pinned three decline paths, two of which no
longer exist (a stopped engine has no clock to hand it a Tick, and there is no retired clock for
one to arrive from); it is now `an_overrun_consumes_no_absolute_tick`, plus the new
`a_requested_stop_prevents_a_tick_before_the_message_arrives`.
`retuning_keeps_the_absolute_tick_of_the_run_it_retunes` read the counter through a test door on
the handle, and there is no longer a way to read it from outside the task — nor a behavioural
proxy, since no Function reads the absolute Tick yet. The rule it pinned is that a retune does not
begin a run, and that is covered: making the `Retune` arm call `begin_run` turns
`retuning_after_a_stall_anchors_on_the_grid_not_the_wake_instant`,
`retuning_anchors_on_the_deadline_a_late_tick_was_due_at` and
`the_native_retuned_clock_holds_the_shared_deadline_rule` red, because `begin_run` clears the last
executed deadline and a grid with nothing to anchor on starts at the retune instant.

New tests: `a_requested_stop_prevents_a_tick_before_the_message_arrives` (the request alone, with
no message behind it, declines five deadlines that come due on time and leaves no Overrun behind
— which is what isolates the flag from the queue), `stop_returns_without_waiting_for_a_tick_and_no_tick_follows_it`
(the handle's half, through the public surface, against an adapter that holds the engine inside a
submission), `an_orderly_shutdown_silences_the_output_without_reporting_a_failure`,
`an_engine_cannot_be_constructed_without_a_runtime`, and
`a_refresh_that_has_not_been_answered_keeps_the_menu_it_had` in the console.
`diagnostics_drain_in_order_and_exactly_once` now states its ordering over a real engine across
two different writers — the handle's start failure and the task's output failure and Overrun —
rather than over a mixture of doors.

**Mutations run, each restored and `git status` checked afterwards.**

- the `stop_requested` check in the Deadline arm disabled — `a_requested_stop_prevents_a_tick_before_the_message_arrives` red (6 Ticks against 1). The first version of that test was green under this mutation because a single five-second advance made every deadline an Overrun; it now advances one period at a time, which is what made it a real test.
- `inner.stop()` at the loop's exit removed — `an_orderly_shutdown_silences_the_output_without_reporting_a_failure` red. The safety action still arrives through the state's own `Drop`; what the explicit stop buys is that an ordinary shutdown is not reported to the user as an unexpected termination.
- the `is_playing` guard on `Start` removed — `start_is_idempotent_and_draining_takes_the_diagnostics` red, after the assertion that a second start into a live run delivers no second immediate Tick was added to it. The mutation was green before that: two starts queued together collapse into one Tick whether or not the second begins a run.
- `PlaybackInner::drop` returning before it reports — `unexpected_engine_termination_stops_playback_and_reports_failure` and `clock_failure_remains_observable_after_output_panics` both red.
- the `Retune` arm calling `begin_run` — the three grid-anchoring tests red, as above.

**Dependencies.** `console` gains `tokio` with `macros` and `rt` as a dev-dependency, declared for
every target rather than beside the non-WASM table, because `check_wasm` compiles the unit tests
for `wasm32-unknown-unknown` and those are the two features that target supports. It was already
in the locked graph through `orcvs`, so nothing entered there. `orcvs` loses `tokio-util`, whose
only user in the workspace was `CancellationToken`; `Cargo.lock` drops it and `futures-sink`
with it. `mise run audit_deps` passes.

**One path is now unreachable and is left standing deliberately.** `PlaybackEngine::retune` can
only fail with `ZeroTickPeriod`, and `Bpm` admits 1 to 15000, whose `delay_ms` ranges from 15
seconds down to 1 millisecond and is never zero — so `Orcvs::set_bpm`'s decline branch and
`PlaybackDiagnostic::RetuneFailure` can no longer be reached from the console. That was already
half true before this ticket; what changed is that `RuntimeUnavailable`, the one retune failure a
caller could stage, moved to construction. The validation is `retune`'s stated contract over a
`Duration` rather than a `Bpm`, so it is kept rather than deleted, and the now-unstageable test
that drove it is the one that moved to construction. Worth a separate look.

**Not settled locally.** `console/tests/wasm.rs` is where this ticket's risk lands — the browser
spawn, the browser clock's yield, and the retune grid across a stall — and it is a merge-tier gate.
The concurrency tests that matter most here are the multi-threaded ones, and the merge tier is
what runs them on a second platform. Both are deferred to CI; the local gates did not settle
either. The WASM target was compiled locally with `-D warnings`, which covers that test target's
compilation and nothing about its behaviour.
