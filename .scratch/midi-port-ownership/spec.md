# The console opens the port; Playback owns the connection

**Status:** ready-for-agent

## Problem Statement

Choosing a MIDI destination can hang Orcvs, and the machinery added to stop it hanging has failed
six consecutive reviews without converging.

A performer opens the MIDI menu and asks for a destination list. On macOS that list can only be
produced on the thread that owns the application run loop — the same thread the console is already
drawing on — because a CoreMIDI client created anywhere else answers a stale port list until the
process restarts. The request does not go to that thread. It is queued to the Playback Engine's
task on a worker thread, which then has to send the work back to the main thread and wait for it.
One question crosses the thread boundary four times.

The wait is what fails. The Playback Engine's task waits on its message channel and its next
deadline together, so every moment it spends blocked on the main thread is a Tick it does not
service: missed Ticks, notes left sounding, and — when the main thread is inside runtime teardown
— a process that never exits. Bounding the wait with a timeout does not resolve this, because a
function that must return a value to its caller has only one fallback available when the timeout
expires: run the operation on the calling thread, which is the thing the mechanism exists to
prevent. Five successive revisions have each moved that defect somewhere new.

The same inversion reaches the performer's screen. Because a discovery answer arrives as a
published value rather than as a reply, the console cannot tell whether the list it is reading is
the answer to the refresh it just asked for, and reconstructs that from a set of tracking flags and
a repaint pump. A refresh can clear the menu's status line as though it succeeded while the device
never connects, and a stale status can permanently suppress the message explaining that the running
Orcvs is gone.

## Solution

The console opens the port. Playback owns the connection.

Device discovery and port opening happen entirely on the thread that can perform them — the
console's main thread, which already owns the run loop. Discovery answers the frame that asked it,
as an ordinary return value, so the menu shows the list or the failure immediately and needs no
tracking flags, no repaint pump and no published destination list. Choosing a destination opens the
connection there too.

What crosses into the Playback Engine is an already-open connection, delivered on the ordered
request queue that already carries every other Playback request. The engine's task remains the sole
owner of the adapter, the connection, the note schedule and lifecycle state: it still performs the
safety action on the outgoing connection before installing the new one, still clears scheduled
notes, still turns a delivery refusal into an ordered Playback diagnostic, and still publishes which
destination is selected for the console to read without blocking.

Nothing waits on anything. The cross-thread bridge, its timeout, its fallback and its poll loop are
deleted rather than corrected, because with discovery on the correct thread there is no boundary
left for them to span.

This restores the boundary ADR 0022 already drew: MIDI device selection belongs to the console,
"because the choice of an output port is user configuration rather than part of a running Orcvs."

## User Stories

1. As a performer, I want the MIDI menu to list destinations the moment I open it, so that choosing an output does not feel like waiting on a background job.
2. As a performer, I want a refresh to show its result in the frame that requested it, so that the list I am reading is never one I have already replaced.
3. As a performer, I want a discovery failure to appear in place of the rows, so that an unavailable MIDI service is visible rather than silent.
4. As a performer, I want a device I plug in after starting Orcvs to appear on the next refresh and be selectable, so that discovery and selection cannot disagree about which devices exist.
5. As a performer, I want selecting a destination to report success only when the port actually opened, so that a cleared status line is not mistaken for a connected device.
6. As a performer, I want a refused connection to say why in the console, so that a failed selection is diagnosable rather than merely ineffective.
7. As a performer, I want the message that the running Orcvs is gone to reach the status line even when an older message is still showing, so that a dead engine is never hidden behind a stale error.
8. As a performer, I want changing destination mid-performance to silence the device I am leaving, so that no device is left holding a note Orcvs can no longer reach.
9. As a performer, I want Ticks to keep their timing while the menu is open, so that browsing destinations does not disturb the run.
10. As a performer, I want enumerating devices never to drop a Tick, so that discovery is not audible.
11. As a performer, I want Orcvs to close when I close the window, so that touching the MIDI menu cannot leave a process that has to be killed.
12. As a performer, I want the selected destination to stay visibly selected across frames, so that the menu always shows where output is going.
13. As a maintainer, I want device discovery to run only on the thread that can perform it correctly, so that a stale port list is impossible by construction rather than by timing.
14. As a maintainer, I want the Playback Engine's task never to block on another thread, so that the guarantee that its loop services deadlines is structural.
15. As a maintainer, I want no mechanism whose failure path violates its own invariant, so that a correct configuration is not a matter of how fast the main thread happens to be.
16. As a maintainer, I want the running Orcvs to remain the sole owner of the connection, the note schedule and lifecycle state, so that ADR 0041's ownership guarantee is unchanged.
17. As a maintainer, I want selection to keep travelling as data on the one ordered queue, so that request ordering relative to start, stop and retune is unchanged.
18. As a maintainer, I want the safety action and scheduled-note cleanup to keep happening inside the engine's task, so that changing destination retains its existing guarantees.
19. As a maintainer, I want MIDI device selection to sit where ADR 0022 assigned it, so that the crate boundary states one rule rather than two.
20. As a maintainer, I want a headless consumer of the Orcvs crate to be unable to call a discovery path that cannot work, so that the absence of a run loop is a compile-time or constructor-level fact rather than a runtime hang.
21. As a maintainer, I want the output adapter to carry no backend type parameter, so that an adapter cannot be constructed into a configuration where its backend is unreachable.
22. As a maintainer, I want the toolkit-free crate to carry no platform dispatch dependency, so that the feature contract that promises a build free of native audio dependencies stays true.
23. As a maintainer, I want the tooling check that enforces that contract to catch the class of dependency rather than an enumerated list, so that the next platform binding does not slip through it.
24. As a maintainer, I want the console to hold no state whose only purpose is guessing whether its own request was answered, so that the menu has no flag that can be left set.
25. As a maintainer, I want the browser target to use the same shape as the native target, so that one description of MIDI selection covers both platforms.
26. As a maintainer, I want the existing MIDI fakes to keep working unchanged, so that the redesign is tested through the seam the suite already uses.
27. As a maintainer, I want the selection handle's request path to answer unavailability on the same terms as its read paths, so that a caller cannot receive success for a request nothing will dequeue.
28. As a maintainer, I want an output-only Playback Engine to reject a MIDI adapter at compile time on every constructor, so that silently dead MIDI output is not reachable through a public path.

## Implementation Decisions

- The console owns the MIDI backend. It calls discovery and port opening directly, on its own
  thread, and receives their results as return values.
- `MidiBackend` is unchanged as an interface. Both of its operations keep their current signatures;
  only the caller changes. This keeps the existing fakes valid and introduces no new seam.
- The MIDI output adapter no longer holds a backend and loses its backend type parameter. It holds
  the current connection, the delivery-failure state and the published selection.
- Selection crosses the seam as a request carrying an already-open connection together with the
  identity of the destination it was opened for. It travels on the existing ordered request queue
  and is dispatched by the engine's task like every other request.
- The engine's task keeps every mutation it performs today on a destination change: the safety
  action on the outgoing connection, scheduled-note cleanup, installation of the new connection,
  and publication of the selected identity.
- Discovery results are no longer published. The published value carries the selected destination
  only. The console's frame reads it without blocking, as now.
- Queue acceptance remains distinct from a successful connection, but the failure it can no longer
  hide is a refused port: the port is opened before the request is sent, so a refusal is returned to
  the caller synchronously and a queued request always carries a connection that opened.
- A delivery refusal on an installed connection remains an ordered Playback diagnostic, unchanged.
- The platform dispatch bridge is deleted: the cross-thread call, its timeout, its slot, its poll
  loop, its main-thread test, and the platform dispatch dependency and its unsafe thread query.
- The cached enumeration client becomes a plain owned value rather than a shared mutable one,
  because it now has one owner on one thread.
- The console's refresh-tracking state is deleted along with the repaint pump that served it:
  discovery answers synchronously, so there is nothing to wait for and nothing to poll.
- The console's status line stops being written from a polled observation. A message from the
  engine and a message from a console-initiated action have distinct precedence, and unavailability
  is never suppressed by an unrelated stale message.
- The selection handle's request path checks availability on the same terms as its read paths, so a
  task that ended while its owner is alive is reported rather than answered with success.
- Every public Playback constructor that produces output-only Playback rejects a MIDI adapter,
  including the engine constructor, not only the application-level one.
- The browser target adopts the same shape: discovery and port opening on the main thread, an open
  connection delivered to the engine.
- The tooling contract's native-dependency check matches the class of platform binding rather than
  an enumerated list of crate names.

## Testing Decisions

A good test here states a behaviour a performer or a caller can observe and would notice the loss
of: a menu that shows a list, a device that receives notes, a Tick that still lands on time, a
request that reports unavailability. It does not assert which thread ran a closure, how long a wait
lasted, or that a private field holds a particular value. Timing-shaped assertions are what the
current design forces and what this one removes; a test that needs to sleep is a sign the seam is
wrong.

- The existing integration test over a running Orcvs and its restricted selection handle is the
  primary seam and stays the highest one: it drives discovery, selection and Playback together and
  proves that the destination the console opened is the connection Playback delivers to.
- The existing MIDI backend fakes are prior art and are reused unchanged. The console's fake backend
  continues to drive the menu's behaviour; the silent fallback backend continues to hold the
  no-native-backend build to what the seam promises.
- The console's menu tests cover: a list rendered from a successful discovery, a failure rendered in
  place of rows, a refused port opening reported without clearing the status line, and an
  unavailable running Orcvs reported even when an older message is showing.
- Engine tests cover destination change through the ordered queue: the safety action reaches the
  outgoing connection, scheduled notes are cleared, the new connection receives subsequent output,
  and the selected identity is published. These extend the existing Playback suite rather than
  forming a new one.
- A test proves a Tick still lands while a destination change is being processed, replacing the
  current design's implicit hope that enumeration is fast.
- A compile-fail test proves an output-only Playback Engine cannot be constructed with a MIDI
  adapter through any public constructor, extending the existing compile-fail coverage.
- Because discovery no longer crosses a thread boundary, the suite gains no test that pumps a run
  loop, no test that sleeps waiting for a main queue, and no test whose result depends on how
  quickly a queue drains. The absence of those is itself the outcome being tested for.
- Native and WASM feature combinations are exercised according to the repository verification
  contract, and the no-default-features build is checked to prove the platform dependency is gone.

## Out of Scope

- Changing MIDI destination identity, display names, or the ordering of discovered destinations.
- Changing the safety action's message set or ordering.
- Changing Playback scheduling, Tick ordering, cancellation, shutdown or diagnostic ordering.
- Changing the Source/Playback seam or anything in the language.
- Making discovery asynchronous or incremental; it is a synchronous call on the thread that owns
  the run loop.
- Hot-plug notification. Discovery remains something the performer asks for.
- Adding MIDI device selection to a platform that has no MIDI service.
- Retaining the cross-thread bridge behind a feature flag or as a fallback path.
- Restructuring the crate split, or moving Playback or MIDI output out of the toolkit-free crate.

## Further Notes

ADR 0022 assigns MIDI device selection to the console, "because the choice of an output port is user
configuration rather than part of a running Orcvs", while assigning Playback and MIDI output to the
toolkit-free crate. ADR 0041 gives the Playback Engine's state one owning task and has the console
read published values rather than ask questions. The two are usually read as being in tension here;
they are not. ADR 0022 governs who chooses and opens a port. ADR 0041 governs who owns it once it is
open and what may mutate it. The design above is the one arrangement that satisfies both, and the
current arrangement is the one that satisfies neither cleanly — it moved port opening into Playback,
then had to move the work back across a thread boundary to make it run.

Only two operations are thread-affine: creating a client and enumerating its ports. Sending on an
open connection is not, which is why output delivery already works from the engine's task today and
continues to. That asymmetry is the whole reason the split falls where it does.

The honest cost is that a headless consumer of the Orcvs crate no longer gets discovery for free; it
supplies its own, on a thread it controls. That is more truthful than the current behaviour, where
such a consumer waits out a timeout and then enumerates on a thread the platform does not support.

An alternative was considered and rejected: keep the backend in the engine and make the crossing
fire-and-forget, with the result posted back onto the engine's own queue as a further request. It
deletes the same bridge and preserves the current ownership arrangement, so it is the fallback if
the boundary above is rejected. It was not chosen because it leaves the console still unable to tell
whether a published list answers its own request, and because it keeps port opening in the crate
that ADR 0022 says does not own it.
