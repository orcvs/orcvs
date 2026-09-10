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

**Status:** ready-for-agent

- [ ] A computation's execution state records whether the Interpreter ran for it and with which
      inputs. No new allocation per Tick: the state record already exists for each computation.
- [ ] Execution returns those states beside the Tick Plan. Planning forwards them. The Source
      discards them. Playback and rendering are unchanged.
- [ ] `interpret` and the thread-local record are deleted, and no `cfg` attribute remains on any
      statement inside a shipped function in the Tick scheduler or executor.
- [ ] The nine tests assert the same two questions they ask today — how many computations were
      interpreted, and which inputs each received — from the returned states.
- [ ] One of those nine, the test that proves each root is told the shared Tick and its own anchor,
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
