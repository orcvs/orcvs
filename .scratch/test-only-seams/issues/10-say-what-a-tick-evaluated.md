# 10 — Say what a Tick evaluated, instead of watching the Interpreter

**What to build:** A Tick reports which computations it interpreted and what inputs each one
received, as ordinary output of execution. Nine tests read that report instead of reading a
thread-local record written by a `cfg(test)` statement inside a shipped function.

Today `interpret` wraps one call to the Interpreter and records its inputs into a thread-local when
the crate is built for test. The wrapper has no other content, so it exists only to hold the
record. Delete it and nothing else changes, which is the sign it is not a module.

The fact those tests want belongs to the executor, not to the Interpreter. The executor already
holds one state record for each computation. Put the fact there, return the states beside the Tick
Plan, and let production discard them.

**Why the tests cannot read this from the Tick Plan.** Three facts have no other witness:

- A rejected Tick discards its writes and its Play Commands. An empty plan cannot tell a Tick where
  nothing ran from a Tick where everything ran and was then discarded.
- A suppressed computation writes nothing. So does a computation that ran and diagnosed. Several
  tests must prove the first and not the second.
- A computation that runs twice writes the same value twice. The Source is identical either way.

**Why this is not the seam this effort deletes elsewhere.** A Tick Plan says what to apply; the
states say what happened. Those are different facts, and the second is the one a console, a
debugger, or a diagnostic view will ask for. An output no caller reads yet is ordinary. An input no
production path can supply is not, and that is what the other tickets remove.

**Blocked by:** 04 — Build a destination-carrying schedule a test can construct.

**Status:** resolved

- [x] A computation's execution state records whether the Interpreter ran for it and with which
      inputs. No new allocation per Tick: the state record already exists for each computation.
- [x] Execution returns those states beside the Tick Plan. Planning forwards them. The Source
      discards them. Playback and rendering are unchanged.
- [x] `interpret` and the thread-local record are deleted, and no `cfg` attribute remains on any
      statement inside a shipped function in the Tick scheduler or executor.
- [x] The nine tests assert the same two questions they ask today — how many computations were
      interpreted, and which inputs each received — from the returned states. Ten tests, not nine:
      see the note below.
- [x] One of those nine, the test that proves each root is told the shared Tick and its own anchor,
      carries a note naming its retirement: when a Function reads a Tick or an anchor, that test
      asserts through the Function's result and stops reading states.

## Comments

Raised while reviewing `03`, 2026-09-10. `03` removed the substitution seam and left this record
standing, correctly — it is a different thing. It changes nothing a Tick computes: the Interpreter
receives the same arguments and answers the same value in both builds.

Sequence this with `05` through `07` rather than before them. Six of the nine tests reach the
executor through the test-only Source execution helper that `08` deletes, so their route changes in
those tickets anyway. Carrying the state change at the same time changes each of them once. Doing
this first changes them twice.

The first cost estimate for this work was wrong and is recorded here so it is not repeated: it
assumed a separate collection filled once per Tick, and therefore a production allocation bought
for a test. Per-computation state is already allocated, so the fact costs a field rather than a
vector.

2026-09-10: Resolved by the commit this line lands in, on one branch with `06` and `07` and after
both of them, as this ticket's own sequencing note asked.

`ComputationState` carries `interpreted: Option<TickInputs>` — the inputs the Interpreter received
for that computation, or `None` where it never ran for it. It is set beside the call in
`take_turn`, after the operands resolve, so a Turn whose operands would not resolve is recorded as
one the Interpreter never ran for. The record costs a field on a struct already allocated once per
computation, as the corrected estimate in this ticket said it would.

`execute` and `Execution::reject` return `(TickPlan, Vec<ComputationState>)`. `plan`,
`plan_configured`, `plan_carrying`, `unscheduled` and `execution::stated::plan_with_answers`
forward the pair; `Source::plan_tick` forwards it and `Source::execute` discards the states.
`Source::execute` still answers a `TickPlan` and nothing outside this module's tests reads a state,
so playback and rendering are untouched.

`interpret` and `mod observed` are deleted. The only `cfg` attributes left in the Tick scheduler
and executor are on items — the test-only routes `carry`, `schedule_carrying`, `plan_carrying`,
`mod test` and `mod stated` — and none is on a statement inside a shipped function.

Ten tests read the record, not nine. The tenth is
`a_carried_destination_schedules_the_tick_a_configured_one_would_have`, which `04` added after this
ticket was written and which compares the two routes' evaluations as well as their plans. All ten
now read `interpreted`, a test helper that keeps the entries whose state has one, and every
expected value is the one the test asserted before.

Two consequences of reading states rather than a thread-local are worth recording:

- The entries are in the order the schedule holds its computations, which is the order they were
  parsed, not the order their Turns were taken. Nine of the ten read only the length or filter for
  one value, so the difference is invisible to them. The tenth,
  `the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor`, asserts an ordered list of
  three anchors; its three roots are independent, so the parsed order and the scheduled order are
  the same list and its expectation is unchanged. Its assertion message no longer claims scheduled
  order, which is the only line of any of the ten that says something different than before.
- The tests no longer clear a record before driving a Tick. `observed::take()` had to be called
  first because the thread outlived the Tick; a returned state cannot carry an earlier Tick's
  evaluation, so eleven such calls simply went away.

`ComputationState::interpreted` carries `#[allow(dead_code)]` with a reason: no shipped caller
reads it, which is the ordinary shape this ticket argues for. `expect` is wrong here because the
method is dead in the library build and live in the test build, so the expectation would go
unfulfilled in the second and fail the clippy gate that compiles both.

The retirement note asked for by the fifth criterion is on
`the_interpreter_is_handed_the_shared_tick_and_each_roots_own_anchor`. It records that the Tick
half is already asserted through results by
`the_tick_functions_answer_about_the_absolute_tick_they_are_planned_at`, and that the anchor half
has no result to be visible in only because no built Function reads its anchor yet — when one
does, that test asserts through the Function's answer and stops reading states.
