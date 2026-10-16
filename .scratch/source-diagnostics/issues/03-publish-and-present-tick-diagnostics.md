# 03 — Publish and present Tick diagnostics

Status: ready-for-agent
Blocked by: 04

**What to build:**

The Playback Engine publishes each executed Tick's diagnostics, replacing the previous Tick's so the observation stays bounded, and the Diagnostics Panel lists them in a section of its own.

## Acceptance criteria

- [ ] `execute_tick` in `orcvs/src/playback.rs` publishes the Tick's diagnostics through the existing observation; nothing buffers without bound.
- [ ] The Diagnostics Panel gains a section, below a separator from the rendering readouts, with a fixed number of rows (three to start) so its height is reserved before Source layout.
- [ ] Rows group diagnostics by message, each showing the message, a count and the first anchor's Position; `+N more` appears when there are more groups than rows.
- [ ] A diagnostic stays listed for a fixed wall-clock time (one second to start) after the last Tick that produced it; a repeat resets its timer. Stopping Playback clears the section.
- [ ] The time and row count are presentation constants, not Playback configuration.
- [ ] Language Map diagnostics are not listed here (ticket 02).
- [ ] Tested through the shipped console under the egui skill, including a Play whose note slot Track empties.

## Comments

### Decided 2026-10-16 by Toby

Provisional: this is a starting point, not the final design; presentation will be iterated once it is in use. It reverses `docked-diagnostics/01`'s note that the Panel shows only rendering readouts. The Panel stays one View → Diagnostics panel rather than splitting into Performance and Diagnostics; marking anchors on the Grid and a scrolling list are out of scope.
