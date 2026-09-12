# 23 — Give a Playback run a type of its own

**What to decide, then build:** The Playback Engine has states and transitions. It carries them as loose fields on `PlaybackInner`, and the rule for what each field means in each state lives in prose beside the transition that happens to touch it. Decide whether a run should be a type, so that "what is true while a run is live" and "what a transition must clear" are checked rather than remembered.

## The states are already there, spelled as flags

`PlaybackInner` (`orcvs/src/playback.rs:614-636`) holds `playing: bool`, `connected: bool`, `generation: u64`, `cancellation: Option<CancellationToken>`, `last_tick_at: Option<ClockInstant>`, `last_output_failure: Option<OutputAdapterError>`, `tick: Tick`, and the note schedule. Not every combination is legal. `cancellation` is `Some` exactly while a run is live; `generation` advances per run; `tick` counts executed Ticks of the current run; `last_tick_at` is meaningless before the first Tick of one. Nothing in the type says any of that.

The transitions are already named methods — `begin_run`, `stop`, `disconnect`, `retune`, `select_destination`, `execute_tick` — plus task death through `ClockRunGuard::drop`. `CONTEXT.md`'s Playback Engine entry writes the table in prose: "Beginning a run, stopping, disconnecting, and changing destination each clear the schedule."

## The rules are per-field and subtle, which is the point

This is not a case of someone forgetting. `begin_run` (`playback.rs:761-789`) resets seven things; `stop` (`playback.rs:710-726`) resets three, and the difference is deliberate and argued. The latch carries eleven lines explaining why it is cleared in `begin_run` "here alone, and deliberately not in `stop`" — because the safety action `stop` sends is the last submission of the run that recorded it, so a refusal repeating that run's failure is the same fault. The note schedule carries its own paragraph for why it clears in both.

Every one of those arguments is correct. All of them live in comments, and each new run-scoped field needs its own such argument written against every transition.

## What went wrong once already

Issue `22` fixed `last_output_failure` outliving the run that recorded it: a device failing the same way in two consecutive runs was reported once, and the second run reported nothing. That is the canonical defect of this shape — a field whose lifetime is a run, in a struct that has no idea what a run is.

Its third acceptance bullet is the reason to open this: "The reset is stated where the other run-scoped inputs are reset, so the next run-scoped input added cannot be reset on one target and forgotten on another." That is the right instinct, and convention is the weakest form of it. The next person adding a run-scoped field gets a hint from where the others sit, and nothing else.

## The question to settle first

**Is a run a value, or a phase of the engine?**

Two shapes answer this differently, and the choice is not obvious:

- **A run is a value.** `PlaybackInner` holds `run: Option<Run>`, where `Run` owns `generation`, `cancellation`, `tick`, `last_tick_at`, `last_output_failure` and the note schedule. `begin_run` constructs one; `stop` drops it. Dropping is what clears everything, so a run-scoped field cannot be forgotten by construction. The cost is that `stop`'s careful "not the latch, because the safety action is still this run's submission" argument has to be re-expressed as ordering — send the safety action, *then* drop — rather than as a per-field exception.
- **A run is a phase.** An explicit `enum` over `Idle`/`Running { .. }`, with the engine matching on it. Same guarantee for the fields inside the variant, but keeps `connected` and the adapter outside, where they belong — an adapter survives a run.

`connected` is the awkward part either way: it is adapter state, not run state, and today it sits in the same struct as both.

**Do not build this until that is answered.** It is a design decision about how lifecycle invariants are expressed, and ADR 0002 already records the decision that the Playback Engine owns lifecycle concurrency — this would refine how, not whether, which may warrant an ADR of its own.

## What must not change

- ADR 0002's decision that lifecycle concurrency stays inside this module, and that the Playback Engine is a cloneable handle exposing no lock, token, task or generation. Issue `20` delivered that and it holds.
- ADR 0001's Source/Playback seam.
- The safety action's content and ordering, and the fact that every lifecycle action that silences output clears the note schedule.
- Every behaviour `22` pinned: one report per run, not one per Tick and not one per adapter lifetime.

## Where this came from

An architecture review of the workspace, in the session that filed `22`. Its argument was that the Tick pipeline is a pipeline and should not become a state machine — nothing persists between its stages — while the Playback Engine is the one part of the system that genuinely has states and events, and is the one modelling them as loose fields.

**Status:** resolved

## Answer

Closed against [ADR 0040](../../../docs/adr/0040-the-playback-engine-owns-its-state-in-one-task.md)
rather than built, and the `playback-actor` effort implemented that decision in
`playback-actor/04`.

The question this issue asked first — is a run a value or a phase of the engine? — was
answered by neither of the two shapes it offered. Both keep the lock and give the state behind
it better types, and ADR 0040 records why that treats the symptom: state behind a lock is opaque
to the type system, which is *why* the invariants had to be written as comments at each mutation
site and why `last_output_failure` could outlive its run until review found it. Ownership is what
makes the ordering expressible, and a task is how a shared handle gets an owner.

What the effort did instead is take the whole of `PlaybackInner` out of `Arc<Mutex<..>>` and make
it the state of one task, reached through a cloneable handle that sends messages. Three of the
four flags this issue listed as untyped are not better typed — they are gone. `generation`,
`cancellation` and `ClockRunGuard` existed only because a second task called into shared state;
with one task there is no second party to be stale relative to, nothing asleep that must be
woken, and no other task whose death must be detected. `cancellation` being `Some` exactly while
a run is live is now the grid existing exactly while a run is live, held by the loop that waits
on it and dropped when the run ends, which is the ownership this issue was reaching for.

What this issue's own framing got right is kept and recorded in the ADR: the transitions are
still named methods, `connected` is still adapter state rather than run state, and `stop` still
sends its safety action before anything is cleared, for the reason argued here — the action is
the last submission of the run that recorded the latch.

Everything under "What must not change" holds. ADR 0002's guarantees are unchanged, including
that further Ticks are prevented before `stop` returns; ADR 0001's Source/Playback seam is
untouched; the safety action's content and ordering are untouched; and issue `22`'s behaviour is
still pinned by
`playback::tests::a_run_that_begins_after_a_failed_run_reports_the_failure_again`.
