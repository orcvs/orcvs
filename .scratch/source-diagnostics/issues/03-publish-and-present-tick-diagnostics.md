# 03 — Publish and present Tick diagnostics

Status: ready-for-agent
Blocked by: none

**What to build:**

The Playback Engine records each executed Tick's diagnostics in its bounded diagnostic log (ADR 0055), and the console lists the ones it drains in a section of the Diagnostics Panel of its own.

## Acceptance criteria

- [ ] `execute_tick` in `orcvs/src/playback.rs` records the Tick's diagnostics as a new `PlaybackDiagnostic` class in the existing diagnostic log, not in `PlaybackObservation`, which stays `Copy`. Diagnostics with the same message are coalesced into one entry per Tick, carrying a count and the first anchor's Position, so a Tick with many broken Expressions does not fill the log.
- [ ] The Tick class is the first evicted, with Overrun, so a broken Expression repeating every Tick never displaces an output, clock, start or retune failure. An ADR amends ADR 0055 with the new class and its place in the eviction order.
- [ ] The console routes drained Tick entries to the Panel; `MidiDeviceSelection::observe_diagnostics` ignores them, as it ignores Overrun.
- [ ] The Diagnostics Panel gains a section, below a separator from the rendering readouts, with a fixed number of rows (three to start) so its height is reserved before Source layout.
- [ ] Rows group diagnostics by message, each showing the message, a count and the first anchor's Position; `+N more` appears when there are more groups than rows.
- [ ] A diagnostic stays listed for a fixed wall-clock time (one second to start) after the drain that last answered it; a repeat resets its timer. Stopping Playback clears the section.
- [ ] The time and row count are presentation constants, not Playback configuration.
- [ ] Language Map diagnostics are not listed here (ticket 02).
- [ ] Tested through the shipped console under the egui skill, including a Play whose note slot Track empties.

## Comments

### Decided 2026-10-16 by Toby

Provisional: this is a starting point, not the final design; presentation will be iterated once it is in use. It reverses `docked-diagnostics/01`'s note that the Panel shows only rendering readouts. The Panel stays one View → Diagnostics panel rather than splitting into Performance and Diagnostics; marking anchors on the Grid and a scrolling list are out of scope.

### Decided 2026-10-16 by Toby

Tick diagnostics travel through the Playback Engine's diagnostic log rather than `PlaybackObservation` or `tracing`. The observation is `Copy` and a `watch` value keeps only the latest Tick, and `tracing` has no subscriber on the browser build and no reader in the Panel. The log is already bounded, drained every frame and seen once per drain. Tick scheduling never visits a stray operand pair, so this ticket does not wait on 04.
