# 03 — The console opens the port; Playback receives an open connection

**What to build:** A performer opens the MIDI menu and the destination list is there in that frame,
produced on the thread that can produce it correctly. They pick a device; the port opens
immediately and either connects or says why it did not. Playback then delivers to that device, and
the device they left is silenced on the way out. Browsing the menu and changing destination no
longer disturb the run: Ticks keep their timing throughout, and closing the window closes the
process even if a refresh was in flight.

The console owns the MIDI backend and calls it directly. What crosses into Playback is an
already-open connection carried on the ordered request queue that already carries start, stop and
retune. The Playback Engine's task remains the sole owner of the connection, the note schedule and
lifecycle state — it simply stops being the thing that opens the port. The cross-thread bridge that
sent this work back to the main thread and waited for it is deleted rather than corrected, along
with its timeout, its slot, its poll loop and its main-thread test, because there is no longer a
boundary for them to span.

This restores the boundary ADR 0022 draws — the choice of an output port is user configuration
rather than part of a running Orcvs — while keeping every ownership guarantee ADR 0041 states.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Discovery returns its answer to the caller that asked for it, on the console's own thread. The
      menu renders the list, or the failure in place of the rows, in the frame that requested it.
- [x] Opening a port happens on the console's thread and reports refusal to the caller
      synchronously. A queued selection request always carries a connection that opened.
- [x] Selection crosses the seam as a request carrying the open connection and the identity of the
      destination it was opened for, on the existing ordered queue, dispatched by the engine's task
      like every other request.
- [x] Changing destination preserves today's guarantees inside the engine's task: the safety action
      reaches the outgoing connection, scheduled notes are cleared, the new connection receives
      subsequent output, and the selected identity is published for the console to read without
      blocking.
- [x] A delivery refusal on an installed connection remains an ordered Playback diagnostic.
- [x] The published value carries the selected destination only; discovery results are no longer
      published.
- [x] The output adapter holds no backend and carries no backend type parameter.
- [x] The cross-thread dispatch bridge is gone: no timeout, no slot, no poll loop, no unsafe
      main-thread query, and no code path that runs a platform call on a thread it chose by
      fallback.
- [x] The cached enumeration client is a plain owned value with one owner on one thread, not shared
      mutable state.
- [ ] A test proves a Tick still lands on time while a destination change is being processed.
      Deferred to 08.
- [x] The existing integration test over a running Orcvs and its restricted selection handle proves
      that the destination the console opened is the connection Playback delivers to.
- [x] The existing MIDI backend fakes drive these tests unchanged; no new test seam is introduced.
- [x] No test sleeps waiting for a queue to drain, pumps a run loop, or depends on how quickly a
      thread responds.
- [x] A headless consumer of the Orcvs crate supplies its own discovery; no public path offers one
      that cannot work without a run loop.
